/**
 * Trading metrics calculations.
 */

import type { FitnessMetrics } from "./types";

/**
 * Calculate the Sharpe ratio of returns.
 *
 * @param returns - Array of periodic returns
 * @param periodsPerYear - Number of periods per year (252 for daily stocks, 365 for crypto)
 * @param riskFreeRate - Risk-free rate (default 0.0)
 * @returns Annualized Sharpe ratio, or NaN if calculation is not possible
 */
export function sharpeRatio(
	returns: number[],
	periodsPerYear: number,
	riskFreeRate = 0,
): number {
	// Filter out NaN values
	const validReturns = returns.filter((r) => !Number.isNaN(r));

	if (validReturns.length < 2) {
		return Number.NaN;
	}

	const n = validReturns.length;
	const mean = validReturns.reduce((a, b) => a + b, 0) / n;

	// Sample standard deviation
	const variance =
		validReturns.reduce((sum, r) => sum + (r - mean) ** 2, 0) / (n - 1);
	const stdDev = Math.sqrt(variance);

	if (stdDev === 0) {
		return Number.NaN;
	}

	const excessReturn = mean - riskFreeRate;
	return Math.sqrt(periodsPerYear) * (excessReturn / stdDev);
}

/**
 * Calculate the maximum drawdown of a NAV series.
 *
 * @param navValues - Array of NAV values
 * @returns Maximum drawdown as a decimal (0.15 = 15% drawdown)
 */
export function maxDrawdown(navValues: number[]): number {
	if (navValues.length === 0) {
		return 0;
	}

	const firstNav = navValues[0];
	if (firstNav === undefined) {
		return 0;
	}

	let runningMax = firstNav;
	let maxDd = 0;

	for (const nav of navValues) {
		if (nav > runningMax) {
			runningMax = nav;
		}
		const drawdown = 1 - nav / runningMax;
		if (drawdown > maxDd) {
			maxDd = drawdown;
		}
	}

	return maxDd;
}

/**
 * Calculate total return from NAV series.
 */
export function totalReturn(navValues: number[]): number {
	if (navValues.length < 2) {
		return 0;
	}
	const first = navValues[0];
	const last = navValues[navValues.length - 1];
	if (first === undefined || last === undefined || first === 0) {
		return 0;
	}
	return (last - first) / first;
}

/**
 * Calculate PnL (returns) from NAV series.
 */
export function pnlFromNav(navValues: number[]): number[] {
	if (navValues.length < 2) {
		return [0];
	}

	const pnl: number[] = [0];
	for (let i = 1; i < navValues.length; i++) {
		const prev = navValues[i - 1];
		const curr = navValues[i];
		if (prev !== undefined && curr !== undefined && prev !== 0) {
			pnl.push((curr - prev) / prev);
		} else {
			pnl.push(0);
		}
	}
	return pnl;
}

/**
 * Calculate complete fitness metrics for a NAV series.
 */
export function calculateFitnessMetrics(
	navValues: number[],
	periodsPerYear: number,
): FitnessMetrics {
	const pnl = pnlFromNav(navValues);
	const sr = sharpeRatio(pnl, periodsPerYear);
	const mdd = maxDrawdown(navValues);
	const total = totalReturn(navValues);

	return {
		sharpeRatio: sr,
		maxDrawdown: mdd,
		totalReturn: total,
		tradingDays: navValues.length,
	};
}
