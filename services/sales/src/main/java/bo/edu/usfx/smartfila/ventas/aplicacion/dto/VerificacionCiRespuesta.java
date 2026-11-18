package bo.edu.usfx.smartfila.ventas.aplicacion.dto;

import java.util.List;

public record VerificacionCiRespuesta(
        boolean valido,
        List<VerificacionComprobanteRespuesta> comprobantes
) {
}
