import { normalize } from "@geolonia/normalize-japanese-addresses";
import fastify from "fastify";

async function main() {
  const server = fastify();
  server.get("/healthz", (_, reply) => {
    reply.code(200).send()
  });
  server.get("/address/:address", async (req, reply) => {
    const { address }: any = req.params;
    const { pref, city, town, addr, level} = await normalize(address)
    reply
      .type('application/json')
      .send({ full: address, pref, city, town, addr, level })
  });
  const port = parseInt(process.env.ADDRESS_VALIDATOR_PORT || '') || 8080;
  await server.listen({ host: "0.0.0.0", port });
  console.log("server is listening at", server.addresses());
}

void main();
