package bo.edu.usfx.smartfila.ventas.infraestructura.persistencia;

import bo.edu.usfx.smartfila.ventas.dominio.AuditoriaInspeccion;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.UUID;

public interface AuditoriaInspeccionRepository
        extends JpaRepository<AuditoriaInspeccion, UUID> {
}
