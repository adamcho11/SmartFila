package bo.edu.usfx.smartfila.ventas.infraestructura.persistencia;

import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;

import java.util.Optional;
import java.util.UUID;

public interface OrdenCobroRepository extends JpaRepository<OrdenCobro, UUID> {
    boolean existsByReservaId(UUID reservaId);

    Page<OrdenCobro> findByCompradorId(UUID compradorId, Pageable pageable);

    Optional<OrdenCobro> findByReferenciaQr(String referenciaQr);

    Optional<OrdenCobro> findByReferenciaPago(String referenciaPago);

    Page<OrdenCobro> findByCompradorIdAndEstado(
            UUID compradorId,
            EstadoOrdenCobro estado,
            Pageable pageable
    );
}
