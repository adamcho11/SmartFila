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
import bo.edu.usfx.smartfila.ventas.dominio.OutboxEvent;
import bo.edu.usfx.smartfila.ventas.infraestructura.mensajeria.RabbitMqEventPublisher;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.AuditoriaPagoRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OutboxEventRepository;
import org.slf4j.MDC;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import bo.edu.usfx.smartfila.ventas.config.VentasMetrics;

@Service
@Transactional(readOnly = true)
public class OrdenCobroService {

    private static final String ROUTING_KEY_VENTA_CONFIRMADA = "venta.confirmada";

    private final OrdenCobroRepository repository;
    private final InventarioClient inventarioClient;
    private final AuditoriaPagoRepository auditoriaRepository;
    private final OutboxEventRepository outboxEventRepository;
    private final RabbitMqEventPublisher eventPublisher;
    private final VentasMetrics metrics;

    public OrdenCobroService(
            OrdenCobroRepository repository,
            InventarioClient inventarioClient,
            AuditoriaPagoRepository auditoriaRepository,
            OutboxEventRepository outboxEventRepository,
            RabbitMqEventPublisher eventPublisher,
            VentasMetrics metrics
    ) {
        this.repository = repository;
        this.inventarioClient = inventarioClient;
        this.auditoriaRepository = auditoriaRepository;
        this.outboxEventRepository = outboxEventRepository;
        this.eventPublisher = eventPublisher;
        this.metrics = metrics;
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
    public OrdenCobroRespuesta crear(
            CrearOrdenCobroSolicitud solicitud,
            UUID compradorSolicitanteId,
            String compradorNombre,
            String compradorCi
    ) {
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

        if (compradorNombre == null || compradorNombre.isBlank()
                || compradorCi == null || compradorCi.isBlank()) {
            throw new org.springframework.security.access.AccessDeniedException(
                    "El JWT no contiene los datos mínimos del comprador"
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
                reserva.venceEn(),
                compradorNombre,
                compradorCi,
                reserva.sector(),
                reserva.unidad()
        );

        OrdenCobroRespuesta respuesta = OrdenCobroRespuesta.desde(repository.save(orden));
        metrics.ordenCreada();
        return respuesta;
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
            metrics.pagoConfirmado();

            UUID eventId = UUID.randomUUID();
            Instant ahora = Instant.now();
            String correlationId = resolverCorrelationId();
            String payload = construirPayloadVentaConfirmada(
                    eventId,
                    ahora,
                    correlationId,
                    guardada,
                    solicitud.referenciaPago(),
                    false
            );
            OutboxEvent outbox = new OutboxEvent(
                    eventId,
                    guardada.getId(),
                    ROUTING_KEY_VENTA_CONFIRMADA,
                    payload,
                    correlationId,
                    ahora
            );
            outboxEventRepository.save(outbox);
            if (eventPublisher.publicar(eventId, ROUTING_KEY_VENTA_CONFIRMADA, payload, correlationId)) {
                outbox.marcarPublicado(Instant.now());
                outboxEventRepository.save(outbox);
            }
        }
        return OrdenCobroRespuesta.desde(guardada);
    }

    @Transactional
    public Map<String, Object> republicarEventoConfirmacion(UUID ordenId) {
        List<OutboxEvent> eventos = outboxEventRepository.findByAggregateIdOrderByCreadoEnAsc(ordenId);
        if (eventos.isEmpty()) {
            throw new RecursoNoEncontradoException(
                    "No existen eventos registrados en outbox para la orden indicada"
            );
        }
        OutboxEvent evento = eventos.get(eventos.size() - 1);
        boolean publicado = eventPublisher.publicar(
                evento.getId(),
                evento.getRoutingKey(),
                evento.getPayload(),
                evento.getCorrelationId()
        );
        if (publicado && evento.getPublicadoEn() == null) {
            evento.marcarPublicado(Instant.now());
            outboxEventRepository.save(evento);
        }

        Map<String, Object> respuesta = new LinkedHashMap<>();
        respuesta.put("eventId", evento.getId());
        respuesta.put("orderId", ordenId);
        respuesta.put("routingKey", evento.getRoutingKey());
        respuesta.put("publicado", publicado);
        respuesta.put("reentregaIdempotente", true);
        return respuesta;
    }

    public Map<String, Object> publicarEventoFalloSimulado(String motivo) {
        UUID eventId = UUID.randomUUID();
        Instant ahora = Instant.now();
        String correlationId = resolverCorrelationId();
        String referencia = (motivo == null || motivo.isBlank())
                ? "sim-dlq-" + eventId
                : motivo.trim();
        String payload = construirPayloadFalloSimulado(eventId, ahora, correlationId, referencia);
        boolean publicado = eventPublisher.publicar(
                eventId,
                ROUTING_KEY_VENTA_CONFIRMADA,
                payload,
                correlationId
        );

        Map<String, Object> respuesta = new LinkedHashMap<>();
        respuesta.put("eventId", eventId);
        respuesta.put("routingKey", ROUTING_KEY_VENTA_CONFIRMADA);
        respuesta.put("simulateFailure", true);
        respuesta.put("publicado", publicado);
        return respuesta;
    }

    private String resolverCorrelationId() {
        String correlationId = MDC.get("correlationId");
        return (correlationId == null || correlationId.isBlank())
                ? UUID.randomUUID().toString()
                : correlationId;
    }

    private String construirPayloadVentaConfirmada(
            UUID eventId,
            Instant occurredAt,
            String correlationId,
            OrdenCobro orden,
            String paymentReference,
            boolean simulateFailure
    ) {
        String unit = orden.getUnidad() == null ? "" : orden.getUnidad();
        return construirJsonEvento(
                eventId,
                occurredAt,
                correlationId,
                orden.getId(),
                orden.getReservaId(),
                orden.getCompradorId(),
                orden.getCompradorCi() == null ? "" : orden.getCompradorCi(),
                orden.getSector() == null ? "" : orden.getSector(),
                unit,
                unit,
                orden.getMonto(),
                paymentReference,
                simulateFailure
        );
    }

    private String construirPayloadFalloSimulado(
            UUID eventId,
            Instant occurredAt,
            String correlationId,
            String paymentReference
    ) {
        return construirJsonEvento(
                eventId,
                occurredAt,
                correlationId,
                UUID.randomUUID(),
                UUID.randomUUID(),
                UUID.randomUUID(),
                "0000000",
                "Sector DLQ",
                "Puesto DLQ",
                "Puesto DLQ",
                new BigDecimal("25.00"),
                paymentReference,
                true
        );
    }

    private String construirJsonEvento(
            UUID eventId,
            Instant occurredAt,
            String correlationId,
            UUID orderId,
            UUID reservationId,
            UUID userId,
            String buyerCi,
            String sector,
            String unit,
            String sku,
            BigDecimal amount,
            String paymentReference,
            boolean simulateFailure
    ) {
        StringBuilder sb = new StringBuilder(512);
        sb.append('{');
        sb.append("\"eventId\":").append(jsonString(eventId.toString())).append(',');
        sb.append("\"eventType\":").append(jsonString(ROUTING_KEY_VENTA_CONFIRMADA)).append(',');
        sb.append("\"occurredAt\":").append(jsonString(occurredAt.toString())).append(',');
        sb.append("\"correlationId\":").append(jsonString(correlationId)).append(',');
        sb.append("\"data\":{");
        sb.append("\"orderId\":").append(jsonString(orderId.toString())).append(',');
        sb.append("\"reservationId\":").append(jsonString(reservationId.toString())).append(',');
        sb.append("\"userId\":").append(jsonString(userId.toString())).append(',');
        sb.append("\"buyerCi\":").append(jsonString(buyerCi)).append(',');
        sb.append("\"sector\":").append(jsonString(sector)).append(',');
        sb.append("\"unit\":").append(jsonString(unit)).append(',');
        sb.append("\"sku\":").append(jsonString(sku)).append(',');
        sb.append("\"amount\":").append(amount.toPlainString()).append(',');
        sb.append("\"currency\":\"BOB\",");
        sb.append("\"paymentReference\":").append(jsonString(paymentReference));
        if (simulateFailure) {
            sb.append(",\"simulateFailure\":true");
        }
        sb.append("}}");
        return sb.toString();
    }

    private static String jsonString(String value) {
        if (value == null) {
            return "\"\"";
        }
        StringBuilder escaped = new StringBuilder(value.length() + 4);
        escaped.append('"');
        for (int i = 0; i < value.length(); i++) {
            char c = value.charAt(i);
            switch (c) {
                case '"' -> escaped.append("\\\"");
                case '\\' -> escaped.append("\\\\");
                case '\b' -> escaped.append("\\b");
                case '\f' -> escaped.append("\\f");
                case '\n' -> escaped.append("\\n");
                case '\r' -> escaped.append("\\r");
                case '\t' -> escaped.append("\\t");
                default -> {
                    if (c < 0x20) {
                        escaped.append(String.format("\\u%04x", (int) c));
                    } else {
                        escaped.append(c);
                    }
                }
            }
        }
        escaped.append('"');
        return escaped.toString();
    }

    @Transactional
    public OrdenCobroRespuesta anular(UUID ordenId) {
        OrdenCobro orden = repository.findById(ordenId)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe la orden de cobro indicada"
                ));
        try {
            if (orden.anular(Instant.now())) {
                repository.save(orden);
                auditoriaRepository.save(new AuditoriaPago(
                        ordenId,
                        "VENTA_ANULADA",
                        orden.getReferenciaPago(),
                        orden.getMonto()
                ));
            }
        } catch (IllegalStateException exception) {
            throw new ConflictoNegocioException(exception.getMessage());
        }
        return OrdenCobroRespuesta.desde(orden);
    }
}
