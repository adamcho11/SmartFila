package bo.edu.usfx.smartfila.ventas.api;

import bo.edu.usfx.smartfila.ventas.aplicacion.InspeccionService;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.VerificacionCiRespuesta;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.VerificacionComprobanteRespuesta;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.security.SecurityRequirement;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.UUID;

@RestController
@RequestMapping("/api/v1/ventas/verificaciones")
@Tag(name = "Inspección", description = "Verificación de comprobantes por inspectores")
@SecurityRequirement(name = "bearerAuth")
public class InspeccionController {

    private final InspeccionService service;

    public InspeccionController(InspeccionService service) {
        this.service = service;
    }

    @GetMapping("/qr/{referenciaQr}")
    @Operation(summary = "Verifica un comprobante por referencia QR")
    public VerificacionComprobanteRespuesta verificarQr(
            @PathVariable String referenciaQr,
            @AuthenticationPrincipal Jwt jwt
    ) {
        return service.verificarQr(referenciaQr, UUID.fromString(jwt.getSubject()));
    }

    @GetMapping("/ci/{ci}")
    @Operation(summary = "Busca comprobantes vigentes por C.I.")
    public VerificacionCiRespuesta verificarCi(
            @PathVariable String ci,
            @AuthenticationPrincipal Jwt jwt
    ) {
        return service.verificarCi(ci, UUID.fromString(jwt.getSubject()));
    }
}
