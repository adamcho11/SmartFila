package bo.edu.usfx.smartfila.ventas.api;

import bo.edu.usfx.smartfila.ventas.api.error.RecursoNoEncontradoException;
import bo.edu.usfx.smartfila.ventas.aplicacion.OrdenCobroService;
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
import org.springframework.web.bind.annotation.*;
import org.springdoc.core.annotations.ParameterObject;
import bo.edu.usfx.smartfila.ventas.api.dto.CrearOrdenCobroSolicitud;

import java.net.URI;
import java.util.Map;
import java.util.UUID;

@RestController
@RequestMapping("/api/v1/ventas")
@Tag(name = "Ventas", description = "Órdenes de cobro del microservicio de Ventas")
public class VentasController {

    private final OrdenCobroService service;

    public VentasController(OrdenCobroService service) {
        this.service = service;
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
            ) @ParameterObject Pageable pageable
    ) {
        return service.listarPorComprador(compradorId, estado, pageable);
    }

    @GetMapping("/referencias-qr/{referenciaQr}")
    public ResponseEntity<OrdenCobroRespuesta> buscarPorQr(
            @PathVariable String referenciaQr
    ) {
        return service.buscarPorReferenciaQr(referenciaQr)
                .map(ResponseEntity::ok)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe una orden con esa referencia QR"
                ));
    }

    @GetMapping("/{id}")
    public ResponseEntity<OrdenCobroRespuesta> buscarPorId(@PathVariable UUID id) {
        return service.buscarPorId(id)
                .map(ResponseEntity::ok)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe una orden de cobro con ese identificador"
                ));
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
            @Valid @RequestBody CrearOrdenCobroSolicitud solicitud
    ) {
        OrdenCobroRespuesta orden = service.crear(solicitud);

        return ResponseEntity
                .created(URI.create("/api/v1/ventas/" + orden.id()))
                .body(orden);
    }
}
