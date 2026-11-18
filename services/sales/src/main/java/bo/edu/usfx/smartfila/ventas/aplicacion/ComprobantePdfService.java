package bo.edu.usfx.smartfila.ventas.aplicacion;

import bo.edu.usfx.smartfila.ventas.api.error.ConflictoNegocioException;
import bo.edu.usfx.smartfila.ventas.api.error.RecursoNoEncontradoException;
import bo.edu.usfx.smartfila.ventas.dominio.EstadoOrdenCobro;
import bo.edu.usfx.smartfila.ventas.dominio.OrdenCobro;
import bo.edu.usfx.smartfila.ventas.infraestructura.persistencia.OrdenCobroRepository;
import com.google.zxing.BarcodeFormat;
import com.google.zxing.EncodeHintType;
import com.google.zxing.WriterException;
import com.google.zxing.client.j2se.MatrixToImageWriter;
import com.google.zxing.qrcode.QRCodeWriter;
import org.apache.pdfbox.pdmodel.PDDocument;
import org.apache.pdfbox.pdmodel.PDPage;
import org.apache.pdfbox.pdmodel.PDPageContentStream;
import org.apache.pdfbox.pdmodel.common.PDRectangle;
import org.apache.pdfbox.pdmodel.font.PDType1Font;
import org.apache.pdfbox.pdmodel.font.Standard14Fonts;
import org.apache.pdfbox.pdmodel.graphics.image.LosslessFactory;
import org.apache.pdfbox.pdmodel.graphics.image.PDImageXObject;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Map;
import java.util.UUID;

@Service
public class ComprobantePdfService {

    private static final DateTimeFormatter FECHA = DateTimeFormatter
            .ofPattern("dd/MM/yyyy HH:mm XXX")
            .withZone(ZoneId.of("America/La_Paz"));

    private final OrdenCobroRepository repository;
    private final String baseUrlVerificacion;

    public ComprobantePdfService(
            OrdenCobroRepository repository,
            @Value("${smartfila.comprobante.verificador-base-url:http://localhost:8084}")
            String baseUrlVerificacion
    ) {
        this.repository = repository;
        this.baseUrlVerificacion = baseUrlVerificacion.replaceAll("/$", "");
    }

    public byte[] generar(UUID ordenId) {
        OrdenCobro orden = repository.findById(ordenId)
                .orElseThrow(() -> new RecursoNoEncontradoException(
                        "No existe la orden de cobro indicada"
                ));

        if (orden.getEstado() != EstadoOrdenCobro.CONFIRMADO) {
            throw new ConflictoNegocioException(
                    "El comprobante solo está disponible después del pago confirmado"
            );
        }
        if (vacio(orden.getCompradorNombre()) || vacio(orden.getCompradorCi())
                || vacio(orden.getSector()) || vacio(orden.getUnidad())) {
            throw new ConflictoNegocioException(
                    "La orden no tiene los datos necesarios para emitir el comprobante"
            );
        }

        String urlQr = baseUrlVerificacion
                + "/api/v1/ventas/verificaciones/qr/" + orden.getReferenciaQr();
        try (PDDocument documento = new PDDocument();
             ByteArrayOutputStream salida = new ByteArrayOutputStream()) {
            PDPage pagina = new PDPage(PDRectangle.A4);
            documento.addPage(pagina);

            PDType1Font normal = new PDType1Font(Standard14Fonts.FontName.HELVETICA);
            PDType1Font negrita = new PDType1Font(Standard14Fonts.FontName.HELVETICA_BOLD);
            try (PDPageContentStream contenido = new PDPageContentStream(documento, pagina)) {
                escribir(contenido, negrita, 20, 50, 785, "SMARTFILA - COMPROBANTE DE VENTA");
                escribir(contenido, normal, 11, 50, 755, "Orden: " + orden.getId());
                escribir(contenido, normal, 11, 50, 735, "Comprador: " + orden.getCompradorNombre());
                escribir(contenido, normal, 11, 50, 715, "C.I.: " + orden.getCompradorCi());
                escribir(contenido, normal, 11, 50, 695, "Sector: " + orden.getSector());
                escribir(contenido, normal, 11, 50, 675, "Unidad: " + orden.getUnidad());
                escribir(contenido, normal, 11, 50, 655, "Monto pagado: Bs. " + orden.getMonto().toPlainString());
                escribir(contenido, normal, 11, 50, 635, "Confirmado: " + FECHA.format(orden.getConfirmadaEn()));
                escribir(contenido, normal, 9, 50, 225, "Escanea el QR para validar el comprobante.");
                escribir(contenido, normal, 8, 50, 210, "El código contiene únicamente una referencia de verificación.");

                var matriz = new QRCodeWriter().encode(
                        urlQr,
                        BarcodeFormat.QR_CODE,
                        240,
                        240,
                        Map.of(EncodeHintType.MARGIN, 1, EncodeHintType.CHARACTER_SET, "UTF-8")
                );
                PDImageXObject imagenQr = LosslessFactory.createFromImage(
                        documento, MatrixToImageWriter.toBufferedImage(matriz)
                );
                contenido.drawImage(imagenQr, 50, 45, 150, 150);
            }

            documento.save(salida);
            return salida.toByteArray();
        } catch (IOException | WriterException excepcion) {
            throw new IllegalStateException("No se pudo generar el comprobante PDF", excepcion);
        }
    }

    private static void escribir(
            PDPageContentStream contenido,
            PDType1Font fuente,
            float tamano,
            float x,
            float y,
            String texto
    ) throws IOException {
        contenido.beginText();
        contenido.setFont(fuente, tamano);
        contenido.newLineAtOffset(x, y);
        contenido.showText(texto);
        contenido.endText();
    }

    private static boolean vacio(String valor) {
        return valor == null || valor.isBlank();
    }
}
