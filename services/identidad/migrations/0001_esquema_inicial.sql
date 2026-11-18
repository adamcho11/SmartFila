-- Identidad · esquema inicial (documento técnico v2, sección 7.1).
-- Todas las tablas usan UUID y fechas timestamptz en UTC.
-- Los nombres de las restricciones UNIQUE los usa el código para traducir la
-- violación al error del catálogo (CI_ALREADY_REGISTERED, EMAIL_ALREADY_REGISTERED).

CREATE TABLE organizers (
    id            uuid PRIMARY KEY,
    name          varchar(120) NOT NULL,
    contact_email varchar(254) NOT NULL,
    status        varchar(16)  NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'DISABLED')),
    created_at    timestamptz  NOT NULL DEFAULT now(),
    CONSTRAINT organizers_name_unico UNIQUE (name)
);

CREATE TABLE users (
    id            uuid PRIMARY KEY,
    ci            varchar(12)  NOT NULL,
    -- '' cuando no hay complemento: en PostgreSQL dos NULL no chocan en un UNIQUE.
    ci_complement varchar(4)   NOT NULL DEFAULT '',
    first_names   varchar(80)  NOT NULL,
    last_names    varchar(80)  NOT NULL,
    email         varchar(254) NOT NULL,
    phone         varchar(16)  NOT NULL,
    status        varchar(16)  NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'DISABLED')),
    created_at    timestamptz  NOT NULL DEFAULT now(),
    updated_at    timestamptz  NOT NULL DEFAULT now(),
    CONSTRAINT users_ci_unico    UNIQUE (ci, ci_complement),
    CONSTRAINT users_email_unico UNIQUE (email)
);

-- Separada de users para que ninguna consulta de perfil traiga el hash por descuido.
CREATE TABLE credentials (
    user_id       uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    password_hash text        NOT NULL, -- Argon2id en formato PHC
    updated_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE user_roles (
    user_id      uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role         varchar(16) NOT NULL CHECK (role IN ('ADMIN', 'ORGANIZADOR', 'INSPECTOR', 'CIUDADANO')),
    organizer_id uuid REFERENCES organizers (id),
    PRIMARY KEY (user_id, role),
    -- RN-08: ORGANIZADOR e INSPECTOR siempre tienen organizador; ADMIN y CIUDADANO nunca.
    CONSTRAINT user_roles_organizador_coherente
        CHECK ((role IN ('ORGANIZADOR', 'INSPECTOR')) = (organizer_id IS NOT NULL))
);
CREATE INDEX user_roles_organizer_idx ON user_roles (organizer_id) WHERE organizer_id IS NOT NULL;

-- RN-07: id = claim sid del JWT. Como mucho una sesión sin revocar por usuario.
CREATE TABLE sessions (
    id         uuid PRIMARY KEY,
    user_id    uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    CHECK (expires_at > created_at)
);
CREATE UNIQUE INDEX sessions_una_activa_por_usuario ON sessions (user_id) WHERE revoked_at IS NULL;

-- RN-26: nace del evento sancion.confirmada (consumidor en un commit siguiente).
CREATE TABLE sanctions (
    id           uuid PRIMARY KEY,
    user_id      uuid        NOT NULL REFERENCES users (id),
    organizer_id uuid        NOT NULL REFERENCES organizers (id),
    incident_id  uuid        NOT NULL,
    reason       text        NOT NULL,
    status       varchar(16) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'LIFTED')),
    created_at   timestamptz NOT NULL DEFAULT now(),
    lifted_at    timestamptz,
    lifted_by    uuid REFERENCES users (id),
    CONSTRAINT sanctions_incidente_unico UNIQUE (incident_id)
);
CREATE INDEX sanctions_activas_idx ON sanctions (organizer_id, user_id) WHERE status = 'ACTIVE';

-- Idempotencia del consumidor de RabbitMQ: el eventId se guarda en la misma
-- transacción que el efecto.
CREATE TABLE processed_messages (
    event_id     uuid PRIMARY KEY,
    processed_at timestamptz NOT NULL DEFAULT now()
);
