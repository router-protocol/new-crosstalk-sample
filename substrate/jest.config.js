/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: "ts-jest",
  // testEnvironment: 'node',
  // testTimeout: 20000,
  globalSetup: "./tests/global/globalSetup.ts",
  globalTeardown: "./tests/global/globalTeardown.ts",
};
