package bo.edu.usfx.smartfila.ventas.dominio;

import jakarta.persistence.*;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

@Entity
@Table(name = "ordenes_cobro", uniqueConstraints = {
        @UniqueConstraint(name = "uk_orden_cobro_reserva", columnNames = "reserva_id")
})
public class OrdenCobro {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @Version
    private long version;

    @Column(name = "reserva_id", nullable = false)
    private UUID reservaId;

    @Column(name = "comprador_id", nullable = false)
    private UUID compradorId;

    @Column(name = "comprador_nombre", length = 160)
    private String compradorNombre;

    @Column(name = "comprador_ci", length = 20)
    private String compradorCi;

    @Column(name = "sector_nombre", length = 100)
    private String sector;

    @Column(name = "unidad_etiqueta", length = 100)
    private String unidad;

    @Column(nullable = false, precision = 12, scale = 2)
    private BigDecimal monto;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private EstadoOrdenCobro estado = EstadoOrdenCobro.PENDIENTE;

    @Column(name = "referencia_qr", nullable = false, unique = true, length = 36)
    private String referenciaQr = UUID.randomUUID().toString();

    @Column(name = "creada_en", nullable = false, updatable = false)
    private Instant creadaEn;

    @Column(name = "vence_en", nullable = false)
    private Instant venceEn;

    @Column(name = "confirmada_en")
    private Instant confirmadaEn;

    @Column(name = "anulada_en")
    private Instant anuladaEn;

    @Column(name = "referencia_pago", unique = true, length = 100)
    private String referenciaPago;

    @Column(name = "contenido_qr", nullable = false, length = 512)
    private String contenidoQr;

    protected OrdenCobro() {
    }

    public OrdenCobro(
            UUID reservaId,
            UUID compradorId,
            BigDecimal monto,
            Instant venceEn
    ) {
        this(reservaId, compradorId, monto, venceEn, null, null, null, null);
    }

    public OrdenCobro(
            UUID reservaId,
            UUID compradorId,
            BigDecimal monto,
            Instant venceEn,
            String compradorNombre,
            String compradorCi,
            String sector,
            String unidad
    ) {
        this.reservaId = reservaId;
        this.compradorId = compradorId;
        this.monto = monto;
        this.venceEn = venceEn;
        this.compradorNombre = compradorNombre;
        this.compradorCi = compradorCi;
        this.sector = sector;
        this.unidad = unidad;
        this.contenidoQr = "SMARTFILA|COBRO|v1|"
                + this.referenciaQr + "|BOB|"
                + monto.stripTrailingZeros().toPlainString() + "|"
                + venceEn;
    }

    @PrePersist
    void antesDeGuardar() {
        creadaEn = Instant.now();
    }

    public UUID getId() {
        return id;
    }

    public UUID getReservaId() {
        return reservaId;
    }

    public UUID getCompradorId() {
        return compradorId;
    }

    public String getCompradorNombre() { return compradorNombre; }
    public String getCompradorCi() { return compradorCi; }
    public String getSector() { return sector; }
    public String getUnidad() { return unidad; }

    public BigDecimal getMonto() {
        return monto;
    }

    public EstadoOrdenCobro getEstado() {
        return estado;
    }

    public String getReferenciaQr() {
        return referenciaQr;
    }

    public Instant getCreadaEn() {
        return creadaEn;
    }

    public Instant getVenceEn() {
        return venceEn;
    }

    public String getContenidoQr() {
        return contenidoQr;
    }

    public void confirmarPago(String referenciaPago, BigDecimal montoConfirmado) {
        if (referenciaPago == null || referenciaPago.isBlank()) {
            throw new IllegalArgumentException("La referencia de pago es obligatoria");
        }

        if (montoConfirmado == null || monto.compareTo(montoConfirmado) != 0) {
            throw new IllegalArgumentException("El monto confirmado no coincide con la orden");
        }

        if (estado == EstadoOrdenCobro.CONFIRMADO) {
            if (referenciaPago.equals(this.referenciaPago)) {
                return;
            }
            throw new IllegalStateException("La orden ya fue confirmada con otro pago");
        }

        if (estado != EstadoOrdenCobro.PENDIENTE) {
            throw new IllegalStateException(
                    "Solo se puede confirmar una orden pendiente"
            );
        }

        estado = EstadoOrdenCobro.CONFIRMADO;
        this.referenciaPago = referenciaPago;
        confirmadaEn = Instant.now();
    }

    public String getReferenciaPago() {
        return referenciaPago;
    }

    public Instant getConfirmadaEn() {
        return confirmadaEn;
    }

    public Instant getAnuladaEn() { return anuladaEn; }

    public boolean anular(Instant ahora) {
        if (estado == EstadoOrdenCobro.ANULADO) return false;
        if (estado != EstadoOrdenCobro.CONFIRMADO) {
            throw new IllegalStateException("Solo se puede anular una venta confirmada");
        }
        estado = EstadoOrdenCobro.ANULADO;
        anuladaEn = ahora;
        return true;
    }

    public boolean expirarSiVencida(Instant ahora) {
        if (estado != EstadoOrdenCobro.PENDIENTE || ahora.isBefore(venceEn)) {
            return false;
        }

        estado = EstadoOrdenCobro.EXPIRADO;
        return true;
    }

}
