package bo.edu.usfx.smartfila.ventas.dominio;

import org.junit.jupiter.api.Test;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.*;

class OrdenCobroTest {

    @Test
    void confirmarPagoEsIdempotente() {
        var orden = new OrdenCobro(
                UUID.randomUUID(),
                UUID.randomUUID(),
                new BigDecimal("25.00"),
                Instant.now().plusSeconds(600)
        );

        orden.confirmarPago("pago-demo-1", new BigDecimal("25.00"));
        Instant primeraConfirmacion = orden.getConfirmadaEn();

        orden.confirmarPago("pago-demo-1", new BigDecimal("25.00"));

        assertEquals(EstadoOrdenCobro.CONFIRMADO, orden.getEstado());
        assertEquals(primeraConfirmacion, orden.getConfirmadaEn());
        assertEquals("pago-demo-1", orden.getReferenciaPago());
    }

    @Test
    void noConfirmaSiElMontoNoCoincide() {
        var orden = new OrdenCobro(
                UUID.randomUUID(),
                UUID.randomUUID(),
                new BigDecimal("25.00"),
                Instant.now().plusSeconds(600)
        );

        assertThrows(
                IllegalArgumentException.class,
                () -> orden.confirmarPago("pago-demo-2", new BigDecimal("24.00"))
        );
        assertEquals(EstadoOrdenCobro.PENDIENTE, orden.getEstado());
    }

    @Test
    void expiraSoloUnaVezCuandoVence() {
        var orden = new OrdenCobro(
                UUID.randomUUID(),
                UUID.randomUUID(),
                new BigDecimal("25.00"),
                Instant.parse("2026-11-18T10:00:00Z")
        );

        Instant despuesDelVencimiento =
                Instant.parse("2026-11-18T10:01:00Z");

        assertTrue(orden.expirarSiVencida(despuesDelVencimiento));
        assertFalse(orden.expirarSiVencida(despuesDelVencimiento));
        assertEquals(EstadoOrdenCobro.EXPIRADO, orden.getEstado());
    }
}
