package bo.edu.usfx.smartfila.ventas.api.dto;

import jakarta.validation.constraints.NotNull;

import java.util.UUID;

public record CrearOrdenCobroSolicitud(
        @NotNull UUID reservaId
) {
}