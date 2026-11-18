import { Producto, Turno } from './modelos';

/*
 * Datos de cada evento que Inventario todavía no guarda: a qué evento
 * pertenece cada producto, su tipo de unidad, el recorrido y la calle de cada
 * tramo. Inventario solo trae nombre, SKU, stock y precio, así que el evento
 * se reconoce por el prefijo del SKU (GUA-A-001 → evento GUA, tramo A).
 * Cuando el backend exponga los eventos, este archivo se reemplaza por esa lectura.
 */

export interface TramoRuta {
  calle: string;
  desde: string;
  hasta: string;
}

/** Tramo: N metros de acera en una calle del recorrido. Asiento: lugar numerado. */
export type TipoUnidad = 'tramo' | 'asiento';

export interface Evento {
  /** Prefijo del SKU sin el guion: GUA, COL… */
  clave: string;
  nombre: string;
  ciudad?: string;
  unidad: TipoUnidad;
  /** Recorrido en orden, de la salida a la llegada (solo eventos con tramos). */
  ruta?: string[];
  /** Calle de cada tramo, por la letra del SKU. */
  tramos?: Record<string, TramoRuta>;
}

/**
 * Recorrido habitual de la Entrada (Correo del Sur, 2022–2026) y sectores de
 * venta de la Alcaldía: la Hernando Siles es el sector más caro y la Germán
 * Mendoza el más barato, igual que los tramos A y B de los datos de prueba.
 */
export const ENTRADA_GUADALUPE: Evento = {
  clave: 'GUA',
  nombre: 'Entrada de la Virgen de Guadalupe',
  ciudad: 'Sucre',
  unidad: 'tramo',
  ruta: [
    'Plazuela Padilla',
    'Av. Germán Mendoza',
    'Av. Venezuela',
    'Av. Hernando Siles',
    'Calle Aniceto Arce',
    'Plaza 25 de Mayo',
  ],
  tramos: {
    A: { calle: 'Av. Hernando Siles', desde: 'TDJ', hasta: 'Aniceto Arce' },
    B: { calle: 'Av. Germán Mendoza', desde: 'El Reloj', hasta: 'Manuel Vilar' },
  },
};

/** Segundo evento de los datos de prueba (COL-PLA-01): venta por asiento. */
export const CONCIERTO_COLISEO: Evento = {
  clave: 'COL',
  nombre: 'Concierto en el Coliseo',
  unidad: 'asiento',
};

export const EVENTOS: Evento[] = [ENTRADA_GUADALUPE, CONCIERTO_COLISEO];

/** El evento que se muestra en la portada y en el fondo. */
export const EVENTO_DESTACADO = ENTRADA_GUADALUPE;

/** Evento al que pertenece un producto según su SKU; null si no es de ninguno. */
export function eventoDe(sku: string): Evento | null {
  return EVENTOS.find((e) => sku.startsWith(`${e.clave}-`)) ?? null;
}

/** "Tramo A-1 (1 m) · Entrada de Guadalupe" → "Tramo A-1 (1 m)". */
export function tituloUnidad(nombre: string): string {
  return nombre.split(' · ')[0];
}

/** Calle del tramo de un producto (GUA-A-001 → Av. Hernando Siles), si se conoce. */
export function tramoDe(evento: Evento, sku: string): TramoRuta | null {
  const letra = letraDelTramo(evento, sku);
  return (letra && evento.tramos?.[letra]) || null;
}

/** Lo que va después de la cantidad: "500 m libres", "200 asientos". */
export function etiquetaDisponible(evento: Evento | null, cantidad: number): string {
  if (evento?.unidad === 'tramo') return 'm libres';
  if (evento?.unidad === 'asiento') return cantidad === 1 ? 'asiento' : 'asientos';
  return 'disp.';
}

export interface GrupoEvento<T> {
  /** null: productos o turnos cuyo SKU no corresponde a ningún evento conocido. */
  evento: Evento | null;
  elementos: T[];
}

/** Agrupa en el orden de EVENTOS; lo que no es de ningún evento va al final. */
export function agruparPorEvento<T>(
  elementos: T[],
  eventoDeElemento: (e: T) => Evento | null,
): GrupoEvento<T>[] {
  const grupos = [...EVENTOS, null].map((evento): GrupoEvento<T> => ({
    evento,
    elementos: elementos.filter((e) => eventoDeElemento(e) === evento),
  }));
  return grupos.filter((g) => g.elementos.length > 0);
}

/*
 * Orden de llegada. La Cola numera con "en espera + 1" para todos los eventos
 * juntos, así que repite números en cuanto alguien es admitido o sale. El id
 * del turno es un UUIDv7: sus primeros 48 bits son la hora de creación, y
 * comparar los ids como texto los ordena por llegada.
 */

/** Para Array.sort: el que llegó primero va antes. */
export function porLlegada(a: Turno, b: Turno): number {
  return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}

type EventoDePuesto = (puestoId: string) => Evento | null;

/**
 * Personas en espera que llegaron antes que este turno para el mismo evento.
 * Si aún no se sabe el evento de los puestos, cuenta a todos.
 */
export function delanteEnEvento(
  turno: Turno,
  turnos: Turno[],
  eventoDePuesto: EventoDePuesto,
): number {
  const evento = eventoDePuesto(turno.puestoId);
  return turnos.filter(
    (t) => t.estado === 'EN_ESPERA' && t.id < turno.id && eventoDePuesto(t.puestoId) === evento,
  ).length;
}

/**
 * Número del turno dentro de su evento, como el de una máquina de turnos:
 * 1 + los que llegaron antes para ese evento, hayan salido o no.
 */
export function numeroEnEvento(
  turno: Turno,
  turnos: Turno[],
  eventoDePuesto: EventoDePuesto,
): number {
  const evento = eventoDePuesto(turno.puestoId);
  return turnos.filter((t) => t.id < turno.id && eventoDePuesto(t.puestoId) === evento).length + 1;
}

export interface TramoDisponible extends TramoRuta {
  letra: string;
  /** null mientras no llegan los datos de Inventario. */
  precio: number | null;
  /** Metros libres: cada producto del tramo es 1 m de acera. */
  metros: number | null;
}

export interface Parada {
  nombre: string;
  tramos: TramoDisponible[];
}

/**
 * Recorrido del evento con la disponibilidad de cada tramo.
 * Sin productos (aún cargando o sin conexión) devuelve los tramos sin cifras.
 */
export function paradasDelEvento(evento: Evento, productos: Producto[] | null): Parada[] {
  const tramos = Object.entries(evento.tramos ?? {}).map(([letra, ruta]): TramoDisponible => {
    const propios = productos?.filter((p) => letraDelTramo(evento, p.sku) === letra);
    return {
      letra,
      ...ruta,
      precio: propios?.length ? Math.min(...propios.map((p) => p.unitPrice)) : null,
      metros: propios ? propios.reduce((suma, p) => suma + p.stock, 0) : null,
    };
  });
  return (evento.ruta ?? []).map((nombre) => ({
    nombre,
    tramos: tramos.filter((t) => t.calle === nombre),
  }));
}

/** GUA-A-001 → "A"; un SKU de otro evento → null. */
function letraDelTramo(evento: Evento, sku: string): string | null {
  const prefijo = `${evento.clave}-`;
  if (!sku.startsWith(prefijo)) return null;
  return sku.slice(prefijo.length).split('-')[0] || null;
}
