package bo.edu.usfx.smartfila.ventas.api;

import bo.edu.usfx.smartfila.ventas.api.dto.ConfirmarPagoSolicitud;
import bo.edu.usfx.smartfila.ventas.aplicacion.OrdenCobroService;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.OrdenCobroRespuesta;
import io.swagger.v3.oas.annotations.Hidden;
import jakarta.validation.Valid;
import org.springframework.context.annotation.Profile;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.UUID;

@Hidden
@Profile("demo")
@RestController
@RequestMapping("/api/v1/ventas")
public class PagoSimuladoController {

    private final OrdenCobroService service;

    public PagoSimuladoController(OrdenCobroService service) {
        this.service = service;
    }

    @PostMapping("/{id}/confirmaciones-pago-simuladas")
    public OrdenCobroRespuesta confirmarPago(
            @PathVariable UUID id,
            @Valid @RequestBody ConfirmarPagoSolicitud solicitud
    ) {
        return service.confirmarPago(id, solicitud);
    }
}
