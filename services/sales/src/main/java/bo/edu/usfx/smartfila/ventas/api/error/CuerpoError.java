package bo.edu.usfx.smartfila.ventas.api.error;

import java.util.List;

public record CuerpoError(
        String code,
        String message,
        List<DetalleError> details,
        String traceId
) {
}