package bo.edu.usfx.smartfila.ventas.api;

import bo.edu.usfx.smartfila.ventas.api.error.RecursoNoEncontradoException;
import bo.edu.usfx.smartfila.ventas.aplicacion.OrdenCobroService;
import bo.edu.usfx.smartfila.ventas.aplicacion.ComprobantePdfService;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.OrdenCobroRespuesta;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.Valid;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.domain.Sort;
import org.springframework.data.web.PageableDefault;
import org.springframework.http.ResponseEntity;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.security.access.AccessDeniedException;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.*;
import org.springdoc.core.annotations.ParameterObject;
import bo.edu.usfx.smartfila.ventas.api.dto.CrearOrdenCobroSolicitud;
import bo.edu.usfx.smartfila.ventas.aplicacion.dto.AuditoriaPagoRespuesta;

import java.net.URI;
import java.util.Map;
import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/api/v1/ventas")
@Tag(name = "Ventas", description = "Órdenes de cobro del microservicio de Ventas")
public class VentasController {

    private final OrdenCobroService service;
    private final ComprobantePdfService comprobantePdfService;

    public VentasController(
            OrdenCobroService service,
            ComprobantePdfService comprobantePdfService
    ) {
        this.service = service;
        this.comprobantePdfService = comprobantePdfService;
    }

    @GetMapping("/ping")
    public Map<String, String> ping() {
        return Map.of("servicio", "ventas-service", "estado", "UP");
    }

    @GetMapping
    @Operation(summary = "Lista las órdenes de cobro de un comprador")
    public Page<OrdenCobroRespuesta> listarPorComprador(
            @RequestParam UUID compradorId,
            @RequestParam(required = false) EstadoOrdenCobro estado,
            @PageableDefault(
                    size = 20,
                    sort = "creadaEn",
                    direction = Sort.Direction.DESC
            ) @ParameterObject Pageable pageable,
            @AuthenticationPrincipal Jwt jwt
    ) {
        validarAccesoComprador(compradorId, jwt);
        return service.listarPorComprador(compradorId, estado, pageable);
    }

    @GetMapping("/referencias-qr/{referenciaQr}")
    public ResponseEntity<OrdenCobroRespuesta> buscarPorQr(
            @PathVariable String referenciaQr,
            @AuthenticationPrincipal Jwt jwt
    ) {
        OrdenCobroRespuesta orden = service.buscarPorReferenciaQr(referenciaQr)
                .map(ResponseEntity::ok)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe una orden con esa referencia QR"
                )).getBody();
        List<String> roles = jwt == null ? null : jwt.getClaimAsStringList("roles");
        if (roles != null && roles.contains("CIUDADANO")) {
            validarAccesoComprador(orden.compradorId(), jwt);
        }
        return ResponseEntity.ok(orden);
    }

    @GetMapping("/{id}")
    public ResponseEntity<OrdenCobroRespuesta> buscarPorId(
            @PathVariable UUID id,
            @AuthenticationPrincipal Jwt jwt
    ) {
        OrdenCobroRespuesta orden = service.buscarPorId(id)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe una orden de cobro con ese identificador"
                ));
        validarAccesoComprador(orden.compradorId(), jwt);
        return ResponseEntity.ok(orden);
    }

    @GetMapping("/{id}/auditoria")
    public List<AuditoriaPagoRespuesta> consultarAuditoria(@PathVariable UUID id) {
        return service.listarAuditoria(id);
    }

    @GetMapping(value = "/{id}/comprobante", produces = MediaType.APPLICATION_PDF_VALUE)
    public ResponseEntity<byte[]> descargarComprobante(
            @PathVariable UUID id,
            @AuthenticationPrincipal Jwt jwt
    ) {
        OrdenCobroRespuesta orden = service.buscarPorId(id)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe una orden de cobro con ese identificador"
                ));
        validarAccesoComprador(orden.compradorId(), jwt);
        byte[] pdf = comprobantePdfService.generar(id);
        return ResponseEntity.ok()
                .contentType(MediaType.APPLICATION_PDF)
                .header(HttpHeaders.CONTENT_DISPOSITION,
                        "attachment; filename=\"comprobante-" + id + ".pdf\"")
                .header(HttpHeaders.CACHE_CONTROL, "no-store")
                .body(pdf);
    }

    private void validarAccesoComprador(UUID compradorId, Jwt jwt) {
        if (jwt == null) return; // Perfil de pruebas; en ejecución normal el filtro exige JWT.
        List<String> roles = jwt.getClaimAsStringList("roles");
        if (roles == null || roles.isEmpty()) {
            throw new AccessDeniedException("El token no contiene roles válidos");
        }
        if (roles.contains("ADMIN") || roles.contains("ORGANIZADOR")) return;
        if (!roles.contains("CIUDADANO")) {
            throw new AccessDeniedException("El rol no puede consultar órdenes de compradores");
        }
        if (!compradorId.toString().equals(jwt.getSubject())) {
            throw new AccessDeniedException("No puedes consultar órdenes de otro comprador");
        }
    }

    @PostMapping
    @Operation(summary = "Crea una orden de cobro para una reserva activa")
    @ApiResponses({
            @ApiResponse(responseCode = "201", description = "Orden de cobro creada"),
            @ApiResponse(responseCode = "400", description = "Solicitud inválida"),
            @ApiResponse(responseCode = "404", description = "Reserva inexistente"),
            @ApiResponse(responseCode = "409", description = "Reserva no activa o vencida"),
            @ApiResponse(responseCode = "503", description = "Inventario no disponible")
    })
    public ResponseEntity<OrdenCobroRespuesta> crear(
            @Valid @RequestBody CrearOrdenCobroSolicitud solicitud,
            @AuthenticationPrincipal Jwt jwt
    ) {
        OrdenCobroRespuesta orden = service.crear(
                solicitud,
                UUID.fromString(jwt.getSubject()),
                jwt.getClaimAsString("name"),
                jwt.getClaimAsString("ci")
        );

        return ResponseEntity
                .created(URI.create("/api/v1/ventas/" + orden.id()))
                .body(orden);
    }
}
