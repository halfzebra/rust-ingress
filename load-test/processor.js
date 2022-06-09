const { faker } = require("@faker-js/faker");

// Inspiration: https://dev.to/brpaz/load-testing-your-applications-with-artillery-4m1p

// Example:
// {
//   "ts": "1530228282",
//   "sender": "testy-test-service",
//   "message": {
//       "foo": "bar",
//       "baz": "bang"
//   },
//   "sent-from-ip": "1.2.3.4",
//   "priority": 2
// }

function toDashCase(str) {
  return str.toLowerCase().replace(/[^a-zA-Z0-9]+(.)/g, (_m, chr) => '-' + chr).trim();
}

function generateMessage(length) {
  const keys = new Array(length).fill(null).map(() => faker.company.bsBuzz());
  const values = new Array(length).fill(null).map(() => faker.company.bsBuzz());
  const res = {};

  for (let [i, key] of keys.entries()) {
    res[key] = values[i];
  }
  return res;
}

function generatePayload(ctx, __ee, next) {
  const payload = {
    "ts": faker.datatype.number({ min: 10, max: 2147483647 }).toString(),
    "sender": `${toDashCase(faker.animal.cat())}-service`,
    "message": generateMessage(faker.datatype.number({ min: 1, max: 10 })),
    "sent-from-ip": faker.internet.ip(),
    "priority": faker.datatype.number({ min: 0, max: 10 }),
  }

  ctx.vars.payload = payload;

  ctx.vars.ts = payload.ts;
  ctx.vars.sender = payload.sender;

  return next();
}

module.exports = { generatePayload };
