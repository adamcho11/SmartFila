package bo.edu.usfx.smartfila.ventas.aplicacion.port;

import bo.edu.usfx.smartfila.ventas.aplicacion.dto.ReservaParaCobro;

import java.util.Optional;
import java.util.UUID;

public interface InventarioClient {

    Optional<ReservaParaCobro> consultarReserva(UUID reservaId, UUID compradorSolicitanteId);
}
