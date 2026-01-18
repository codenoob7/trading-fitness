/**
 * Tests for trading metrics calculations.
 */

import { describe, expect, test } from "bun:test";
import {
	calculateFitnessMetrics,
	maxDrawdown,
	pnlFromNav,
	sharpeRatio,
	totalReturn,
} from "./metrics";

describe("sharpeRatio", () => {
	test("returns NaN for empty array", () => {
		expect(sharpeRatio([], 252)).toBeNaN();
	});

	test("returns NaN for single value", () => {
		expect(sharpeRatio([0.01], 252)).toBeNaN();
	});

	test("returns NaN for zero standard deviation", () => {
		expect(sharpeRatio([0.01, 0.01, 0.01], 252)).toBeNaN();
	});

	test("calculates positive Sharpe ratio", () => {
		const returns = [0.01, 0.02, 0.015, 0.01, 0.025];
		const result = sharpeRatio(returns, 252);
		expect(result).toBeGreaterThan(0);
	});

	test("calculates negative Sharpe ratio for negative returns", () => {
		const returns = [-0.01, -0.02, -0.015, -0.01, -0.025];
		const result = sharpeRatio(returns, 252);
		expect(result).toBeLessThan(0);
	});

	test("applies risk-free rate correctly", () => {
		const returns = [0.01, 0.02, 0.015, 0.01, 0.025];
		const withoutRf = sharpeRatio(returns, 252, 0);
		const withRf = sharpeRatio(returns, 252, 0.001);
		expect(withRf).toBeLessThan(withoutRf);
	});

	test("filters NaN values", () => {
		const returns = [0.01, Number.NaN, 0.02, Number.NaN, 0.015];
		const result = sharpeRatio(returns, 252);
		expect(result).not.toBeNaN();
	});
});

describe("maxDrawdown", () => {
	test("returns 0 for empty array", () => {
		expect(maxDrawdown([])).toBe(0);
	});

	test("returns 0 for single value", () => {
		expect(maxDrawdown([100])).toBe(0);
	});

	test("returns 0 for monotonically increasing", () => {
		expect(maxDrawdown([100, 110, 120, 130])).toBe(0);
	});

	test("calculates correct drawdown", () => {
		// Peak at 100, drops to 80 = 20% drawdown
		const nav = [100, 110, 100, 80, 90];
		const result = maxDrawdown(nav);
		expect(result).toBeCloseTo(1 - 80 / 110, 10);
	});

	test("finds maximum drawdown", () => {
		// Multiple drawdowns, largest is 30%
		const nav = [100, 90, 95, 100, 70, 80];
		const result = maxDrawdown(nav);
		expect(result).toBeCloseTo(0.3, 10);
	});
});

describe("totalReturn", () => {
	test("returns 0 for empty array", () => {
		expect(totalReturn([])).toBe(0);
	});

	test("returns 0 for single value", () => {
		expect(totalReturn([100])).toBe(0);
	});

	test("calculates positive return", () => {
		const nav = [100, 110, 120];
		expect(totalReturn(nav)).toBeCloseTo(0.2, 10);
	});

	test("calculates negative return", () => {
		const nav = [100, 90, 80];
		expect(totalReturn(nav)).toBeCloseTo(-0.2, 10);
	});

	test("handles zero first value", () => {
		expect(totalReturn([0, 100])).toBe(0);
	});
});

describe("pnlFromNav", () => {
	test("returns [0] for empty array", () => {
		expect(pnlFromNav([])).toEqual([0]);
	});

	test("returns [0] for single value", () => {
		expect(pnlFromNav([100])).toEqual([0]);
	});

	test("calculates periodic returns", () => {
		const nav = [100, 110, 105];
		const pnl = pnlFromNav(nav);
		expect(pnl.length).toBe(3);
		expect(pnl[0]).toBe(0);
		expect(pnl[1]).toBeCloseTo(0.1, 10);
		expect(pnl[2]).toBeCloseTo(-0.0455, 3);
	});

	test("handles zero previous value", () => {
		const nav = [0, 100];
		const pnl = pnlFromNav(nav);
		expect(pnl[1]).toBe(0);
	});
});

describe("calculateFitnessMetrics", () => {
	test("calculates all metrics", () => {
		const nav = [100, 105, 110, 108, 115, 120];
		const metrics = calculateFitnessMetrics(nav, 252);

		expect(metrics.sharpeRatio).not.toBeNaN();
		expect(metrics.maxDrawdown).toBeGreaterThanOrEqual(0);
		expect(metrics.totalReturn).toBeCloseTo(0.2, 10);
		expect(metrics.tradingDays).toBe(6);
	});

	test("handles edge case with flat NAV", () => {
		const nav = [100, 100, 100];
		const metrics = calculateFitnessMetrics(nav, 252);

		expect(metrics.sharpeRatio).toBeNaN();
		expect(metrics.maxDrawdown).toBe(0);
		expect(metrics.totalReturn).toBe(0);
	});
});
