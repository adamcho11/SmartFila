package bo.edu.usfx.smartfila.ventas.aplicacion.dto;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;
import com.fasterxml.jackson.annotation.JsonProperty;

public record ReservaParaCobro(
        @JsonProperty("reservationId") UUID reservaId,
        @JsonProperty("buyerId") UUID compradorId,
        @JsonProperty("amount") BigDecimal monto,
        @JsonProperty("expiresAt") Instant venceEn,
        @JsonProperty("active") boolean activa
) {
}
