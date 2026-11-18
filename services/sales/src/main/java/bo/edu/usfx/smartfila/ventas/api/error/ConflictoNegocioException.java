package bo.edu.usfx.smartfila.ventas.api.error;

public class ConflictoNegocioException extends RuntimeException {

    public ConflictoNegocioException(String message) {
        super(message);
    }
}