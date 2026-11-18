package bo.edu.usfx.smartfila.ventas.dominio;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

import java.time.Instant;
import java.util.UUID;

@Entity
@Table(name = "outbox_events")
public class OutboxEvent {

    @Id
    private UUID id;

    @Column(name = "aggregate_id", nullable = false)
    private UUID aggregateId;

    @Column(name = "routing_key", nullable = false, length = 100)
    private String routingKey;

    @Column(nullable = false, columnDefinition = "TEXT")
    private String payload;

    @Column(name = "correlation_id", length = 100)
    private String correlationId;

    @Column(name = "creado_en", nullable = false, updatable = false)
    private Instant creadoEn;

    @Column(name = "publicado_en")
    private Instant publicadoEn;

    protected OutboxEvent() {
    }

    public OutboxEvent(
            UUID id,
            UUID aggregateId,
            String routingKey,
            String payload,
            String correlationId,
            Instant creadoEn
    ) {
        this.id = id;
        this.aggregateId = aggregateId;
        this.routingKey = routingKey;
        this.payload = payload;
        this.correlationId = correlationId;
        this.creadoEn = creadoEn;
    }

    public void marcarPublicado(Instant instante) {
        this.publicadoEn = instante;
    }

    public UUID getId() {
        return id;
    }

    public UUID getAggregateId() {
        return aggregateId;
    }

    public String getRoutingKey() {
        return routingKey;
    }

    public String getPayload() {
        return payload;
    }

    public String getCorrelationId() {
        return correlationId;
    }

    public Instant getCreadoEn() {
        return creadoEn;
    }

    public Instant getPublicadoEn() {
        return publicadoEn;
    }
}
