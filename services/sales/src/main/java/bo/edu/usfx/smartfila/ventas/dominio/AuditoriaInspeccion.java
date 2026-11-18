package bo.edu.usfx.smartfila.ventas.dominio;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.PrePersist;
import jakarta.persistence.Table;

import java.time.Instant;
import java.util.UUID;

@Entity
@Table(name = "auditoria_inspecciones")
public class AuditoriaInspeccion {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @Column(name = "orden_id")
    private UUID ordenId;

    @Column(name = "inspector_id", nullable = false)
    private UUID inspectorId;

    @Column(nullable = false, length = 10)
    private String metodo;

    @Column(nullable = false)
    private boolean valido;

    @Column(name = "consultada_en", nullable = false, updatable = false)
    private Instant consultadaEn;

    protected AuditoriaInspeccion() { }

    public AuditoriaInspeccion(
            UUID ordenId,
            UUID inspectorId,
            String metodo,
            boolean valido
    ) {
        this.ordenId = ordenId;
        this.inspectorId = inspectorId;
        this.metodo = metodo;
        this.valido = valido;
    }

    @PrePersist
    void antesDeGuardar() { consultadaEn = Instant.now(); }

    public UUID getId() { return id; }
    public UUID getOrdenId() { return ordenId; }
    public UUID getInspectorId() { return inspectorId; }
    public String getMetodo() { return metodo; }
    public boolean isValido() { return valido; }
    public Instant getConsultadaEn() { return consultadaEn; }
}
