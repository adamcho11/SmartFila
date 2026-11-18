package bo.edu.usfx.smartfila.ventas.infraestructura.persistencia;

import bo.edu.usfx.smartfila.ventas.dominio.OutboxEvent;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;
import java.util.UUID;

public interface OutboxEventRepository extends JpaRepository<OutboxEvent, UUID> {

    List<OutboxEvent> findByAggregateIdOrderByCreadoEnAsc(UUID aggregateId);
}
