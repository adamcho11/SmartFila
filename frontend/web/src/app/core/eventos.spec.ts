import {
  CONCIERTO_COLISEO,
  ENTRADA_GUADALUPE,
  agruparPorEvento,
  delanteEnEvento,
  eventoDe,
  numeroEnEvento,
  paradasDelEvento,
} from './eventos';
import { Producto, Turno } from './modelos';

const producto = (sku: string, stock: number, unitPrice: number): Producto => ({
  id: sku,
  name: sku,
  sku,
  stock,
  unitPrice,
});

describe('paradasDelEvento', () => {
  it('suma los metros de cada tramo y los ubica en su calle del recorrido', () => {
    const paradas = paradasDelEvento(ENTRADA_GUADALUPE, [
      producto('GUA-A-001', 500, 50),
      producto('GUA-A-002', 500, 50),
      producto('GUA-B-001', 480, 40),
      producto('COL-PLA-01', 200, 120), // otro evento: se ignora
    ]);

    expect(paradas.map((p) => p.nombre)).toEqual(ENTRADA_GUADALUPE.ruta);
    const siles = paradas.find((p) => p.nombre === 'Av. Hernando Siles')!;
    expect(siles.tramos).toEqual([
      expect.objectContaining({ letra: 'A', precio: 50, metros: 1000 }),
    ]);
    const mendoza = paradas.find((p) => p.nombre === 'Av. Germán Mendoza')!;
    expect(mendoza.tramos[0]).toEqual(expect.objectContaining({ letra: 'B', metros: 480 }));
    expect(paradas.find((p) => p.nombre === 'Av. Venezuela')!.tramos).toEqual([]);
  });

  it('sin datos deja los tramos sin cifras y sin productos los marca en cero', () => {
    const cargando = paradasDelEvento(ENTRADA_GUADALUPE, null).flatMap((p) => p.tramos);
    expect(cargando.every((t) => t.metros === null && t.precio === null)).toBe(true);

    const vacio = paradasDelEvento(ENTRADA_GUADALUPE, []).flatMap((p) => p.tramos);
    expect(vacio.every((t) => t.metros === 0)).toBe(true);
  });
});

describe('eventos', () => {
  it('reconoce el evento por el prefijo del SKU', () => {
    expect(eventoDe('GUA-B-001')).toBe(ENTRADA_GUADALUPE);
    expect(eventoDe('COL-PLA-01')).toBe(CONCIERTO_COLISEO);
    expect(eventoDe('Puesto D-001')).toBeNull();
  });

  it('agrupa en el orden de los eventos y deja lo desconocido al final', () => {
    const grupos = agruparPorEvento(
      [
        producto('Puesto D-001', 9, 25),
        producto('COL-PLA-01', 200, 120),
        producto('GUA-A-001', 500, 50),
      ],
      (p) => eventoDe(p.sku),
    );
    expect(grupos.map((g) => g.evento?.clave ?? null)).toEqual(['GUA', 'COL', null]);
  });

  it('cuenta delante solo a quienes esperan para el mismo evento', () => {
    const evento: Record<string, string> = { a: 'GUA-A-001', c: 'COL-PLA-01' };
    const turno = (
      posicion: number,
      puestoId: string,
      estado: Turno['estado'] = 'EN_ESPERA',
    ): Turno => ({
      id: `${posicion}`,
      usuarioId: 'u',
      puestoId,
      posicion,
      estado,
    });
    const turnos = [
      turno(1, 'a'),
      turno(2, 'c'),
      turno(3, 'c'),
      turno(4, 'a', 'LIBERADO'),
      turno(5, 'a'),
    ];
    const mio = turnos[4];
    const eventoDePuesto = (id: string) => eventoDe(evento[id]);
    expect(delanteEnEvento(mio, turnos, eventoDePuesto)).toBe(1);
    // Número dentro del evento: cuenta también al que salió (turno 4).
    expect(numeroEnEvento(mio, turnos, eventoDePuesto)).toBe(3);
    expect(numeroEnEvento(turnos[2], turnos, eventoDePuesto)).toBe(2);
  });

  it('ordena por el id (UUIDv7) aunque la Cola repita la posición', () => {
    const a: Turno = {
      id: '01a1eec9-0001-7000-8000-000000000000',
      usuarioId: 'u',
      puestoId: 'p',
      posicion: 3,
      estado: 'EN_ESPERA',
    };
    const b: Turno = { ...a, id: '01a1eeca-0001-7000-8000-000000000000', posicion: 3 };
    const mio: Turno = { ...a, id: '01a1eecb-0001-7000-8000-000000000000', posicion: 2 };
    expect(delanteEnEvento(mio, [a, b, mio], () => null)).toBe(2);
  });
});
