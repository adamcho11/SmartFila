package bo.edu.usfx.smartfila.ventas.aplicacion.dto;

import bo.edu.usfx.smartfila.ventas.dominio.AuditoriaPago;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

public record AuditoriaPagoRespuesta(
        UUID id,
        UUID ordenId,
        String evento,
        String referenciaPago,
        BigDecimal monto,
        Instant registradaEn
) {
    public static AuditoriaPagoRespuesta desde(AuditoriaPago auditoria) {
        return new AuditoriaPagoRespuesta(
                auditoria.getId(), auditoria.getOrdenId(), auditoria.getEvento(),
                auditoria.getReferenciaPago(), auditoria.getMonto(), auditoria.getRegistradaEn()
        );
    }
}
