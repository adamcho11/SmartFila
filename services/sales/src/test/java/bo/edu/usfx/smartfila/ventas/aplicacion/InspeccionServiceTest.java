package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.api.dto.ConfirmarPagoSolicitud;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.AuditoriaInspeccionRepository;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.*;

@SpringBootTest
@ActiveProfiles("test")
@Transactional
class InspeccionServiceTest {

    @Autowired
    private InspeccionService inspeccionService;

    @Autowired
    private OrdenCobroService ordenCobroService;

    @Autowired
    private OrdenCobroRepository ordenRepository;

    @Autowired
    private AuditoriaInspeccionRepository auditoriaRepository;

    @Test
    void validaQrCiYRegistraAuditoriaSinGuardarCiEnClaro() {
        UUID compradorId = UUID.randomUUID();
        UUID inspectorId = UUID.randomUUID();
        OrdenCobro orden = ordenRepository.saveAndFlush(new OrdenCobro(
                UUID.randomUUID(), compradorId, new BigDecimal("25.00"),
                Instant.now().plusSeconds(600), "Ana Pérez", "1234567-1A",
                "Sector A", "Puesto A-12"
        ));
        ordenCobroService.confirmarPago(
                orden.getId(), new ConfirmarPagoSolicitud("pago-inspeccion-1", new BigDecimal("25.00"))
        );

        var porQr = inspeccionService.verificarQr(orden.getReferenciaQr(), inspectorId);
        var porCi = inspeccionService.verificarCi("1234567-1A", inspectorId);

        assertTrue(porQr.valido());
        assertEquals("Ana Pérez", porQr.compradorNombre());
        assertTrue(porCi.valido());
        assertEquals(1, porCi.comprobantes().size());
        assertEquals(2, auditoriaRepository.count());
        assertEquals("QR", auditoriaRepository.findAll().get(0).getMetodo());

        ordenCobroService.anular(orden.getId());
        var anulada = inspeccionService.verificarQr(orden.getReferenciaQr(), inspectorId);
        assertFalse(anulada.valido());
        assertEquals(EstadoOrdenCobro.ANULADO, anulada.estado());
        assertNull(anulada.compradorCi());
    }
}
