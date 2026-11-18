package bo.edu.usfx.smartfila.ventas.infraestructura.http;

import bo.edu.usfx.smartfila.ventas.aplicacion.dto.ReservaParaCobro;
import bo.edu.usfx.smartfila.ventas.aplicacion.port.InventarioClient;
import org.springframework.context.annotation.Profile;
import org.springframework.stereotype.Component;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;

@Component
@Profile("demo")
public class InventarioClientDemo implements InventarioClient {

    @Override
    public Optional<ReservaParaCobro> consultarReserva(UUID reservaId) {
        var reserva = new ReservaParaCobro(
                reservaId,
                UUID.fromString("11111111-1111-1111-1111-111111111111"),
                new BigDecimal("25.00"),
                Instant.now().plusSeconds(600),
                true
        );

        return Optional.of(reserva);
    }
}