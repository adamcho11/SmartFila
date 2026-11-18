package bo.edu.usfx.smartfila.ventas.api;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.context.TestPropertySource;
import org.springframework.test.web.servlet.MockMvc;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
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
class ContratoOpenApiIntegracionTest {

    @Autowired
    private MockMvc mockMvc;

    @Test
    void contratoPublicaRutasYRespuestasClaveDeVentas() throws Exception {
        mockMvc.perform(get("/v3/api-docs"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.openapi").value("3.1.0"))
                .andExpect(jsonPath("$.paths['/api/v1/ventas'].get.responses['200']").exists())
                .andExpect(jsonPath("$.paths['/api/v1/ventas'].post.responses['201']").exists())
                .andExpect(jsonPath("$.paths['/api/v1/ventas/{id}/comprobante'].get.responses['200']").exists())
                .andExpect(jsonPath("$.paths['/api/v1/ventas/{id}/auditoria'].get.responses['200']").exists())
                .andExpect(jsonPath("$.paths['/api/v1/ventas/{id}/anulaciones'].post.responses['200']").exists())
                .andExpect(jsonPath("$.paths['/api/v1/ventas/verificaciones/qr/{referenciaQr}'].get.responses['200']").exists())
                .andExpect(jsonPath("$.components.securitySchemes.bearerAuth.type").value("http"));
    }
}
