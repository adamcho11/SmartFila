package bo.edu.usfx.smartfila.ventas.api.error;

import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.MethodArgumentNotValidException;
import org.springframework.web.bind.MissingServletRequestParameterException;
import org.springframework.web.method.annotation.MethodArgumentTypeMismatchException;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;
import org.springframework.web.client.RestClientException;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.orm.ObjectOptimisticLockingFailureException;
import org.springframework.security.access.AccessDeniedException;

import java.util.List;
import java.util.UUID;

@RestControllerAdvice
public class GlobalExceptionHandler {

    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ResponseEntity<CuerpoError> manejarValidacion(
            MethodArgumentNotValidException exception
    ) {
        List<DetalleError> detalles = exception.getBindingResult()
                .getFieldErrors()
                .stream()
                .map(error -> new DetalleError(
                        error.getField(),
                        error.getDefaultMessage() == null
                                ? "valor inválido"
                                : error.getDefaultMessage()
                ))
                .toList();

        return error(HttpStatus.BAD_REQUEST, "VALIDATION_ERROR", "Datos inválidos", detalles);
    }

    @ExceptionHandler(MethodArgumentTypeMismatchException.class)
    public ResponseEntity<CuerpoError> manejarTipoInvalido(
            MethodArgumentTypeMismatchException exception
    ) {
        return error(
                HttpStatus.BAD_REQUEST,
                "VALIDATION_ERROR",
                "Parámetro inválido",
                List.of(new DetalleError(exception.getName(), "formato inválido"))
        );
    }

    @ExceptionHandler(MissingServletRequestParameterException.class)
    public ResponseEntity<CuerpoError> manejarParametroFaltante(
            MissingServletRequestParameterException exception
    ) {
        return error(
                HttpStatus.BAD_REQUEST,
                "VALIDATION_ERROR",
                "Falta un parámetro obligatorio",
                List.of(new DetalleError(exception.getParameterName(), "es obligatorio"))
        );
    }

    private ResponseEntity<CuerpoError> error(
            HttpStatus status,
            String code,
            String message,
            List<DetalleError> details
    ) {
        String traceId = UUID.randomUUID().toString();
        CuerpoError cuerpo = new CuerpoError(code, message, details, traceId);

        return ResponseEntity.status(status)
                .header("X-Correlation-Id", traceId)
                .body(cuerpo);
    }

    @ExceptionHandler(RecursoNoEncontradoException.class)
    public ResponseEntity<CuerpoError> manejarNoEncontrado(
            RecursoNoEncontradoException exception
    ) {
        return error(
                HttpStatus.NOT_FOUND,
                "RESOURCE_NOT_FOUND",
                exception.getMessage(),
                List.of()
        );
    }

    @ExceptionHandler(ConflictoNegocioException.class)
    public ResponseEntity<CuerpoError> manejarConflicto(
            ConflictoNegocioException exception
    ) {
        return error(
                HttpStatus.CONFLICT,
                "BUSINESS_RULE_VIOLATION",
                exception.getMessage(),
                List.of()
        );
    }

    @ExceptionHandler(DataIntegrityViolationException.class)
    public ResponseEntity<CuerpoError> manejarRestriccionUnica(
            DataIntegrityViolationException exception
    ) {
        return error(
                HttpStatus.CONFLICT,
                "BUSINESS_RULE_VIOLATION",
                "La operación entra en conflicto con una orden ya registrada",
                List.of()
        );
    }

    @ExceptionHandler(ObjectOptimisticLockingFailureException.class)
    public ResponseEntity<CuerpoError> manejarActualizacionConcurrente(
            ObjectOptimisticLockingFailureException exception
    ) {
        return error(
                HttpStatus.CONFLICT,
                "CONCURRENT_UPDATE",
                "La orden cambió durante la operación; vuelve a consultar su estado",
                List.of()
        );
    }

    @ExceptionHandler(AccessDeniedException.class)
    public ResponseEntity<CuerpoError> manejarAccesoDenegado(AccessDeniedException exception) {
        return error(
                HttpStatus.FORBIDDEN,
                "ACCESS_DENIED",
                "No tienes permiso para consultar este recurso",
                List.of()
        );
    }

    @ExceptionHandler(RestClientException.class)
    public ResponseEntity<CuerpoError> manejarErrorDeInventario(
            RestClientException exception
    ) {
        return error(
                HttpStatus.SERVICE_UNAVAILABLE,
                "DEPENDENCY_UNAVAILABLE",
                "No se pudo consultar el servicio de Inventario",
                List.of()
        );
    }
}
