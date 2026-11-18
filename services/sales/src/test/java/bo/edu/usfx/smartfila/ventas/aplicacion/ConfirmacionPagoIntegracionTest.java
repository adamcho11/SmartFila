package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.api.dto.ConfirmarPagoSolicitud;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.AuditoriaPagoRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.assertEquals;

@SpringBootTest
@ActiveProfiles("test")
@Transactional
class ConfirmacionPagoIntegracionTest {

    @Autowired
    private OrdenCobroService service;

    @Autowired
    private OrdenCobroRepository ordenRepository;

    @Autowired
    private AuditoriaPagoRepository auditoriaRepository;

    @Test
    void callbackDuplicadoConfirmaUnaSolaVezYDejaUnaAuditoria() {
        UUID ordenId = ordenRepository.saveAndFlush(new OrdenCobro(
                UUID.randomUUID(), UUID.randomUUID(), new BigDecimal("25.00"),
                Instant.now().plusSeconds(600)
        )).getId();
        var callback = new ConfirmarPagoSolicitud("pago-idempotente-1", new BigDecimal("25.00"));

        var primeraRespuesta = service.confirmarPago(ordenId, callback);
        var segundaRespuesta = service.confirmarPago(ordenId, callback);

        assertEquals(EstadoOrdenCobro.CONFIRMADO, primeraRespuesta.estado());
        assertEquals(primeraRespuesta.confirmadaEn(), segundaRespuesta.confirmadaEn());
        assertEquals(1, auditoriaRepository.findByOrdenIdOrderByRegistradaEnAsc(ordenId).size());
    }
}
