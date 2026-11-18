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

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.UUID;

import static org.springframework.security.test.web.servlet.request.SecurityMockMvcRequestPostProcessors.jwt;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.content;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.header;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

@SpringBootTest
@AutoConfigureMockMvc
@ActiveProfiles("demo")
@TestPropertySource(properties = {
        "spring.datasource.url=jdbc:h2:mem:ventas-pdf-flow-test;DB_CLOSE_DELAY=-1",
        "spring.datasource.username=sa",
        "spring.datasource.password=",
        "spring.jpa.hibernate.ddl-auto=create-drop"
})
class FlujoComprobanteIntegracionTest {

    @Autowired
    private MockMvc mockMvc;

    @Test
    void ciudadanoCompraConfirmaPagoYDescargaComprobante() throws Exception {
        UUID compradorId = UUID.randomUUID();
        var identidad = jwt().jwt(token -> token
                        .subject(compradorId.toString())
                        .claim("roles", List.of("CIUDADANO"))
                        .claim("name", "Ana Pérez")
                        .claim("ci", "1234567-1A"))
                .authorities(new SimpleGrantedAuthority("ROLE_CIUDADANO"));

        var alta = mockMvc.perform(post("/api/v1/ventas")
                        .with(identidad)
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"reservaId\":\"" + UUID.randomUUID() + "\"}"))
                .andExpect(status().isCreated())
                .andReturn();
        String ordenId = com.jayway.jsonpath.JsonPath.read(
                alta.getResponse().getContentAsString(), "$.id"
        );

        mockMvc.perform(post("/api/v1/ventas/" + ordenId + "/confirmaciones-pago-simuladas")
                        .with(identidad)
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"referenciaPago\":\"pago-flujo-demo-01\",\"monto\":25.00}"))
                .andExpect(status().isOk());

        byte[] comprobante = mockMvc.perform(get("/api/v1/ventas/" + ordenId + "/comprobante")
                        .with(identidad))
                .andExpect(status().isOk())
                .andExpect(content().contentType(MediaType.APPLICATION_PDF))
                .andExpect(header().string("Cache-Control", "no-store"))
                .andReturn().getResponse().getContentAsByteArray();

        org.junit.jupiter.api.Assertions.assertEquals(
                "%PDF-",
                new String(comprobante, 0, 5, StandardCharsets.US_ASCII)
        );
    }
}
