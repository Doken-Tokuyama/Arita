# Política de seguridad

## Cómo informar de una vulnerabilidad

Si crees haber encontrado una vulnerabilidad de seguridad en ARITA (el compilador, la CLI, el código Rust que emite o cualquier otro componente de este repositorio), **no abras una incidencia pública**. Escribe a:

**security@exyonq.org**

Incluye, si puedes:

- una descripción del problema y su impacto potencial;
- los pasos para reproducirlo (idealmente un fichero `.arita` mínimo y el comando usado);
- la versión o el *commit* afectado y tu entorno (sistema operativo, versión de Rust).

## Qué puedes esperar

- Acusaremos recibo de tu mensaje en un plazo razonable.
- Evaluaremos el informe y te mantendremos al tanto de la corrección.
- Te pedimos que nos des un plazo razonable para corregir el problema antes de divulgarlo públicamente.

## Alcance

Son relevantes, por ejemplo, los fallos que permitan que el código Rust emitido contenga `unsafe`, que una comprobación del compilador pueda eludirse de forma que se acepte código incorrecto, o que la CLI ejecute acciones no previstas con una entrada manipulada.

## Contacto general

Para cualquier otra consulta: **contact@exyonq.org**.

Consulta también el [README](README.md).
