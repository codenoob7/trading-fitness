/**
 * Tests for ITH (Investment Time Horizon) calculations.
 */

import { describe, expect, test } from "bun:test";
import { determineTmaeg, excessGainExcessLoss } from "./ith";
import { maxDrawdown } from "./metrics";

describe("excessGainExcessLoss", () => {
	test("returns empty result for empty array", () => {
		const result = excessGainExcessLoss([], 0.05);
		expect(result.excessGains).toEqual([]);
		expect(result.excessLosses).toEqual([]);
		expect(result.numOfIthEpochs).toBe(0);
		expect(result.ithEpochs).toEqual([]);
		expect(result.ithIntervalsCv).toBeNaN();
	});

	test("returns valid result for single value", () => {
		const result = excessGainExcessLoss([100], 0.05);
		expect(result.excessGains.length).toBe(1);
		expect(result.excessLosses.length).toBe(1);
		expect(result.numOfIthEpochs).toBe(0);
	});

	test("detects ITH epochs when excess gain exceeds hurdle", () => {
		// Create NAV that rises significantly
		const nav = [100, 105, 110, 115, 120, 125];
		const result = excessGainExcessLoss(nav, 0.05);

		expect(result.excessGains.length).toBe(nav.length);
		expect(result.excessLosses.length).toBe(nav.length);
		expect(result.numOfIthEpochs).toBeGreaterThan(0);
	});

	test("calculates excess gains correctly", () => {
		const nav = [100, 110, 120];
		const result = excessGainExcessLoss(nav, 0.05);

		// First value should be 0
		expect(result.excessGains[0]).toBe(0);
		// Subsequent values should reflect gain from endorsing crest
		expect(result.excessGains[1]).toBeGreaterThan(0);
	});

	test("calculates excess losses correctly", () => {
		const nav = [100, 90, 80];
		const result = excessGainExcessLoss(nav, 0.05);

		// Losses should be positive (representing drawdown)
		expect(result.excessLosses[2]).toBeGreaterThan(0);
	});

	test("calculates ITH intervals CV", () => {
		// Create NAV with multiple ITH epochs
		const nav = [100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200];
		const result = excessGainExcessLoss(nav, 0.05);

		// If multiple epochs detected, CV should be a number
		if (result.numOfIthEpochs >= 2) {
			expect(result.ithIntervalsCv).not.toBeNaN();
		}
	});
});

describe("determineTmaeg", () => {
	test("returns max drawdown for mdd method", () => {
		const nav = [100, 110, 100, 80, 90];
		const tmaeg = determineTmaeg(nav, "mdd", 0.1);
		const mdd = maxDrawdown(nav);
		expect(tmaeg).toBeCloseTo(mdd, 10);
	});

	test("returns fixed value for fixed method", () => {
		const nav = [100, 110, 100, 80, 90];
		const fixedValue = 0.15;
		const tmaeg = determineTmaeg(nav, "fixed", fixedValue);
		expect(tmaeg).toBe(fixedValue);
	});

	test("returns fixed value for unknown method", () => {
		const nav = [100, 110, 100, 80, 90];
		const fixedValue = 0.2;
		const tmaeg = determineTmaeg(nav, "unknown", fixedValue);
		expect(tmaeg).toBe(fixedValue);
	});
});
