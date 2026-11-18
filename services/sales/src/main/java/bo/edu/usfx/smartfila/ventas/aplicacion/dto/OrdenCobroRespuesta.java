package bo.edu.usfx.smartfila.ventas.aplicacion.dto;

import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

public record OrdenCobroRespuesta(
        UUID id,
        UUID reservaId,
        BigDecimal monto,
        EstadoOrdenCobro estado,
        String referenciaQr,
        String contenidoQr,
        Instant creadaEn,
        Instant venceEn,
        Instant confirmadaEn
) {
    public static OrdenCobroRespuesta desde(OrdenCobro orden) {
        return new OrdenCobroRespuesta(
                orden.getId(),
                orden.getReservaId(),
                orden.getMonto(),
                orden.getEstado(),
                orden.getReferenciaQr(),
                orden.getContenidoQr(),
                orden.getCreadaEn(),
                orden.getVenceEn(),
                orden.getConfirmadaEn()
        );
    }
}
