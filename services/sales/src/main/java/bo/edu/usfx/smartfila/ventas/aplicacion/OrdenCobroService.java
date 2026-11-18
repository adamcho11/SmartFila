package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.api.dto.CrearOrdenCobroSolicitud;
import bo.edu.usfx.smartfila.ventas.api.dto.ConfirmarPagoSolicitud;
import bo.edu.usfx.smartfila.ventas.api.error.ConflictoNegocioException;
import bo.edu.usfx.smartfila.ventas.api.error.RecursoNoEncontradoException;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.ReservaParaCobro;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.OrdenCobroRespuesta;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.AuditoriaPagoRespuesta;
import bo.edu.usfx.smartfila.ventas.dominio.AuditoriaPago;
import bo.edu.usfx.smartfila.ventas.aplicacion.port.InventarioClient;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.AuditoriaPagoRepository;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.Instant;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

@Service
@Transactional(readOnly = true)
public class OrdenCobroService {

    private final OrdenCobroRepository repository;
    private final InventarioClient inventarioClient;
    private final AuditoriaPagoRepository auditoriaRepository;

    public OrdenCobroService(
            OrdenCobroRepository repository,
            InventarioClient inventarioClient,
            AuditoriaPagoRepository auditoriaRepository
    ) {
        this.repository = repository;
        this.inventarioClient = inventarioClient;
        this.auditoriaRepository = auditoriaRepository;
    }

    public Page<OrdenCobroRespuesta> listarPorComprador(
            UUID compradorId,
            EstadoOrdenCobro estado,
            Pageable pageable
    ) {
        var pagina = estado == null
                ? repository.findByCompradorId(compradorId, pageable)
                : repository.findByCompradorIdAndEstado(compradorId, estado, pageable);

        return pagina.map(OrdenCobroRespuesta::desde);
    }

    public Optional<OrdenCobroRespuesta> buscarPorReferenciaQr(String referenciaQr) {
        return repository.findByReferenciaQr(referenciaQr)
                .map(OrdenCobroRespuesta::desde);
    }

    public Optional<OrdenCobroRespuesta> buscarPorId(UUID id) {
        return repository.findById(id)
                .map(OrdenCobroRespuesta::desde);
    }

    public List<AuditoriaPagoRespuesta> listarAuditoria(UUID ordenId) {
        if (!repository.existsById(ordenId)) {
            throw new RecursoNoEncontradoException("No existe la orden de cobro indicada");
        }
        return auditoriaRepository.findByOrdenIdOrderByRegistradaEnAsc(ordenId)
                .stream().map(AuditoriaPagoRespuesta::desde).toList();
    }

    @Transactional
    public OrdenCobroRespuesta crear(CrearOrdenCobroSolicitud solicitud, UUID compradorSolicitanteId) {
        if (repository.existsByReservaId(solicitud.reservaId())) {
            throw new ConflictoNegocioException(
                    "La reserva ya tiene una orden de cobro"
            );
        }

        ReservaParaCobro reserva = inventarioClient.consultarReserva(
                        solicitud.reservaId(), compradorSolicitanteId
                )
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe la reserva indicada"
                ));

        if (!compradorSolicitanteId.equals(reserva.compradorId())) {
            throw new org.springframework.security.access.AccessDeniedException(
                    "La reserva no pertenece al comprador autenticado"
            );
        }

        if (!reserva.activa()
                || reserva.venceEn() == null
                || !reserva.venceEn().isAfter(Instant.now())) {
            throw new ConflictoNegocioException(
                    "La reserva no está activa o ya venció"
            );
        }

        if (reserva.monto() == null || reserva.monto().signum() <= 0) {
            throw new ConflictoNegocioException(
                    "Inventario devolvió un monto inválido"
            );
        }

        OrdenCobro orden = new OrdenCobro(
                reserva.reservaId(),
                reserva.compradorId(),
                reserva.monto(),
                reserva.venceEn()
        );

        return OrdenCobroRespuesta.desde(repository.save(orden));
    }

    @Transactional(noRollbackFor = ConflictoNegocioException.class)
    public OrdenCobroRespuesta confirmarPago(UUID ordenId, ConfirmarPagoSolicitud solicitud) {
        OrdenCobro orden = repository.findById(ordenId)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe la orden de cobro indicada"
                ));

        repository.findByReferenciaPago(solicitud.referenciaPago())
                .filter(otraOrden -> !otraOrden.getId().equals(ordenId))
                .ifPresent(otraOrden -> {
                    throw new ConflictoNegocioException(
                            "La referencia de pago ya fue utilizada"
                    );
                });

        if (orden.expirarSiVencida(Instant.now())) {
            repository.save(orden);
            throw new ConflictoNegocioException("La orden de cobro ya venció");
        }

        boolean eraPendiente = orden.getEstado() == EstadoOrdenCobro.PENDIENTE;
        try {
            orden.confirmarPago(solicitud.referenciaPago(), solicitud.monto());
        } catch (IllegalArgumentException | IllegalStateException exception) {
            throw new ConflictoNegocioException(exception.getMessage());
        }

        OrdenCobro guardada = repository.save(orden);
        if (eraPendiente) {
            auditoriaRepository.save(new AuditoriaPago(
                    ordenId, "PAGO_CONFIRMADO", solicitud.referenciaPago(), solicitud.monto()
            ));
        }
        return OrdenCobroRespuesta.desde(guardada);
    }
}
