package bo.edu.usfx.smartfila.ventas.dominio;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.PrePersist;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.UUID;

@Entity
@Table(name = "auditoria_pagos", uniqueConstraints = {
        @UniqueConstraint(name = "uk_auditoria_orden_evento", columnNames = {"orden_id", "evento"})
})
public class AuditoriaPago {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @Column(name = "orden_id", nullable = false)
    private UUID ordenId;

    @Column(nullable = false, length = 40)
    private String evento;

    @Column(name = "referencia_pago", nullable = false, length = 100)
    private String referenciaPago;

    @Column(nullable = false, precision = 12, scale = 2)
    private BigDecimal monto;

    @Column(name = "registrada_en", nullable = false, updatable = false)
    private Instant registradaEn;

    protected AuditoriaPago() {
    }

    public AuditoriaPago(UUID ordenId, String evento, String referenciaPago, BigDecimal monto) {
        this.ordenId = ordenId;
        this.evento = evento;
        this.referenciaPago = referenciaPago;
        this.monto = monto;
    }

    @PrePersist
    void antesDeGuardar() {
        registradaEn = Instant.now();
    }

    public UUID getId() { return id; }
    public UUID getOrdenId() { return ordenId; }
    public String getEvento() { return evento; }
    public String getReferenciaPago() { return referenciaPago; }
    public BigDecimal getMonto() { return monto; }
    public Instant getRegistradaEn() { return registradaEn; }
}
