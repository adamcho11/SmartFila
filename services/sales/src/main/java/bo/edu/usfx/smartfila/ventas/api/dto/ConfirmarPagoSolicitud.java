package bo.edu.usfx.smartfila.ventas.api.dto;

import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

import java.math.BigDecimal;

public record ConfirmarPagoSolicitud(
        @NotBlank @Size(max = 100) String referenciaPago,
        @NotNull @DecimalMin("0.01") BigDecimal monto
) {
}
