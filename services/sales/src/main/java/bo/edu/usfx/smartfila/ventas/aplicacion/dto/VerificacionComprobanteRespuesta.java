package bo.edu.usfx.smartfila.ventas.aplicacion.dto;

import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;

import java.time.Instant;
import java.util.UUID;

public record VerificacionComprobanteRespuesta(
        boolean valido,
        String motivo,
        UUID ordenId,
        EstadoOrdenCobro estado,
        String compradorNombre,
        String compradorCi,
        String sector,
        String unidad,
        Instant confirmadaEn
) {
    public static VerificacionComprobanteRespuesta desde(OrdenCobro orden) {
        boolean vigente = orden.getEstado() == EstadoOrdenCobro.CONFIRMADO;
        return new VerificacionComprobanteRespuesta(
                vigente,
                vigente ? "COMPROBANTE_VALIDO" : "COMPROBANTE_NO_VIGENTE",
                orden.getId(),
                orden.getEstado(),
                vigente ? orden.getCompradorNombre() : null,
                vigente ? orden.getCompradorCi() : null,
                vigente ? orden.getSector() : null,
                vigente ? orden.getUnidad() : null,
                vigente ? orden.getConfirmadaEn() : null
        );
    }

    public static VerificacionComprobanteRespuesta noEncontrado() {
        return new VerificacionComprobanteRespuesta(
                false, "QR_NO_ENCONTRADO", null, null, null, null, null, null, null
        );
    }
}
