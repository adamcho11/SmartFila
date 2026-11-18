package bo.edu.usfx.smartfila.ventas.infraestructura.persistencia;

import bo.edu.usfx.smartfila.ventas.dominio.AuditoriaPago;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface AuditoriaPagoRepository extends JpaRepository<AuditoriaPago, UUID> {
    List<AuditoriaPago> findByOrdenIdOrderByRegistradaEnAsc(UUID ordenId);
}
