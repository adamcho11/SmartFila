package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.api.dto.ConfirmarPagoSolicitud;
import bo.edu.usfx.smartfila.ventas.api.error.ConflictoNegocioException;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import com.google.zxing.BinaryBitmap;
import com.google.zxing.MultiFormatReader;
import com.google.zxing.client.j2se.BufferedImageLuminanceSource;
import com.google.zxing.common.HybridBinarizer;
import org.apache.pdfbox.Loader;
import org.apache.pdfbox.pdmodel.graphics.image.PDImageXObject;
import org.apache.pdfbox.text.PDFTextStripper;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;
import java.util.stream.StreamSupport;

import static org.junit.jupiter.api.Assertions.*;

@SpringBootTest
@ActiveProfiles("test")
@Transactional
class ComprobantePdfServiceTest {

    @Autowired
    private OrdenCobroRepository ordenRepository;

    @Autowired
    private OrdenCobroService ordenCobroService;

    @Autowired
    private ComprobantePdfService comprobantePdfService;

    @Test
    void emitePdfConDatosDeVentaYQrSinDatosPersonales() throws Exception {
        OrdenCobro orden = ordenRepository.saveAndFlush(new OrdenCobro(
                UUID.randomUUID(), UUID.randomUUID(), new BigDecimal("25.00"),
                Instant.now().plusSeconds(600), "Ana Pérez", "1234567-1A",
                "Sector A", "Puesto A-12"
        ));
        UUID id = orden.getId();
        ordenCobroService.confirmarPago(
                id, new ConfirmarPagoSolicitud("pago-pdf-01", new BigDecimal("25.00"))
        );

        byte[] pdf = comprobantePdfService.generar(id);
        assertEquals("%PDF-", new String(pdf, 0, 5, java.nio.charset.StandardCharsets.US_ASCII));

        try (var documento = Loader.loadPDF(pdf)) {
            String texto = new PDFTextStripper().getText(documento);
            assertTrue(texto.contains("Ana Pérez"));
            assertTrue(texto.contains("1234567-1A"));
            assertTrue(texto.contains("Sector A"));
            assertTrue(texto.contains("Puesto A-12"));

            var nombres = documento.getPage(0).getResources().getXObjectNames();
            var imagenQr = StreamSupport.stream(nombres.spliterator(), false)
                    .map(nombre -> {
                        try {
                            return documento.getPage(0).getResources().getXObject(nombre);
                        } catch (java.io.IOException exception) {
                            throw new IllegalStateException(exception);
                        }
                    })
                    .filter(PDImageXObject.class::isInstance)
                    .map(PDImageXObject.class::cast)
                    .findFirst()
                    .orElseThrow();
            String contenidoQr = new MultiFormatReader().decode(new BinaryBitmap(
                    new HybridBinarizer(new BufferedImageLuminanceSource(imagenQr.getImage()))
            )).getText();
            assertTrue(contenidoQr.contains(orden.getReferenciaQr()));
            assertFalse(contenidoQr.contains("1234567"));
            assertFalse(contenidoQr.contains("Ana Pérez"));
        }
    }

    @Test
    void noEmiteComprobanteDeOrdenPendiente() {
        OrdenCobro orden = ordenRepository.saveAndFlush(new OrdenCobro(
                UUID.randomUUID(), UUID.randomUUID(), new BigDecimal("25.00"),
                Instant.now().plusSeconds(600), "Ana Pérez", "1234567-1A",
                "Sector A", "Puesto A-12"
        ));

        assertThrows(
                ConflictoNegocioException.class,
                () -> comprobantePdfService.generar(orden.getId())
        );
    }
}
