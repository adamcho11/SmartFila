package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.aplicacion.dto.VerificacionCiRespuesta;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.VerificacionComprobanteRespuesta;
import bo.edu.usfx.smartfila.ventas.dominio.AuditoriaInspeccion;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.AuditoriaInspeccionRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;
import bo.edu.usfx.smartfila.ventas.config.VentasMetrics;

@Service
public class InspeccionService {

    private final OrdenCobroRepository ordenRepository;
    private final AuditoriaInspeccionRepository auditoriaRepository;
    private final VentasMetrics metrics;

    public InspeccionService(
            OrdenCobroRepository ordenRepository,
            AuditoriaInspeccionRepository auditoriaRepository,
            VentasMetrics metrics
    ) {
        this.ordenRepository = ordenRepository;
        this.auditoriaRepository = auditoriaRepository;
        this.metrics = metrics;
    }

    @Transactional
    public VerificacionComprobanteRespuesta verificarQr(String referenciaQr, UUID inspectorId) {
        var orden = ordenRepository.findByReferenciaQr(referenciaQr);
        boolean valido = orden.map(o -> o.getEstado() == EstadoOrdenCobro.CONFIRMADO)
                .orElse(false);
        auditoriaRepository.save(new AuditoriaInspeccion(
                orden.map(o -> o.getId()).orElse(null),
                inspectorId,
                "QR",
                valido
        ));
        metrics.inspeccion("QR", valido);
        return orden.map(VerificacionComprobanteRespuesta::desde)
                .orElseGet(VerificacionComprobanteRespuesta::noEncontrado);
    }

    @Transactional
    public VerificacionCiRespuesta verificarCi(String ci, UUID inspectorId) {
        var ordenes = ordenRepository.findByCompradorCiAndEstadoOrderByCreadaEnDesc(
                ci, EstadoOrdenCobro.CONFIRMADO
        );
        auditoriaRepository.save(new AuditoriaInspeccion(
                null, inspectorId, "CI", !ordenes.isEmpty()
        ));
        metrics.inspeccion("CI", !ordenes.isEmpty());
        var comprobantes = ordenes.stream()
                .map(VerificacionComprobanteRespuesta::desde)
                .toList();
        return new VerificacionCiRespuesta(!comprobantes.isEmpty(), comprobantes);
    }

}
