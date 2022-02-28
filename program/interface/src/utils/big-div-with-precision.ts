import BN from 'bn.js';
import {Constants} from '../constants';

export function bigDivWithPrecisionHelper(numerator: BN, denominator: BN): BN {
  const decimalPrecisionBN = new BN(Constants.decimalPrecision);
  return numerator.mul(decimalPrecisionBN).div(denominator);
}

export function bigDivWithPrecision(numerator: BN, denominator: BN): number {
  return (
    bigDivWithPrecisionHelper(numerator, denominator).toNumber() /
    Constants.decimalPrecision
  );
}
