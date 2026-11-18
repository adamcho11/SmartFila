package bo.edu.usfx.smartfila.ventas.infraestructura.http;

import bo.edu.usfx.smartfila.ventas.aplicacion.dto.ReservaParaCobro;
import bo.edu.usfx.smartfila.ventas.aplicacion.port.InventarioClient;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.springframework.web.client.HttpClientErrorException;
import org.springframework.web.client.RestClient;


import java.util.Optional;
import java.util.UUID;

import org.springframework.context.annotation.Profile;

@Profile("!demo")
@Component
public class InventarioClientHttp implements InventarioClient {

    private final RestClient restClient;

    public InventarioClientHttp(
            @Value("${smartfila.inventory.base-url:http://localhost:8082}")
            String baseUrl
    ) {
        this.restClient = RestClient.builder()
                .baseUrl(baseUrl)
                .build();
    }

    @Override
    public Optional<ReservaParaCobro> consultarReserva(UUID reservaId, UUID compradorSolicitanteId) {
        try {
            return Optional.ofNullable(
                    restClient.get()
                            .uri("/api/Reservations/{id}", reservaId)
                            .retrieve()
                            .body(ReservaParaCobro.class)
            );
        } catch (HttpClientErrorException.NotFound exception) {
            return Optional.empty();
        }
    }
}
