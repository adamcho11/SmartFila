package bo.edu.usfx.smartfila.ventas.api;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.MediaType;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.context.TestPropertySource;
import org.springframework.test.web.servlet.MockMvc;

import java.util.List;
import java.util.UUID;

import static org.springframework.security.test.web.servlet.request.SecurityMockMvcRequestPostProcessors.jwt;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

@SpringBootTest
@AutoConfigureMockMvc
@ActiveProfiles("demo")
@TestPropertySource(properties = {
        "spring.datasource.url=jdbc:h2:mem:ventas-security-test;DB_CLOSE_DELAY=-1",
        "spring.datasource.username=sa",
        "spring.datasource.password=",
        "spring.jpa.hibernate.ddl-auto=create-drop"
})
class SeguridadVentasIntegracionTest {

    @Autowired
    private MockMvc mockMvc;

    @Test
    void apiDeVentasRequiereJwt() throws Exception {
        mockMvc.perform(get("/api/v1/ventas")
                        .param("compradorId", UUID.randomUUID().toString()))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void ciudadanoNoConsultaOrdenesDeOtraPersona() throws Exception {
        UUID compradorAutenticado = UUID.randomUUID();
        mockMvc.perform(get("/api/v1/ventas")
                        .param("compradorId", UUID.randomUUID().toString())
                        .with(jwt().jwt(token -> token
                                .subject(compradorAutenticado.toString())
                                .claim("roles", List.of("CIUDADANO")))
                                .authorities(new SimpleGrantedAuthority("ROLE_CIUDADANO"))))
                .andExpect(status().isForbidden());
    }

    @Test
    void pingDeDisponibilidadEsPublico() throws Exception {
        mockMvc.perform(get("/api/v1/ventas/ping").accept(MediaType.APPLICATION_JSON))
                .andExpect(status().isOk());
    }
}
