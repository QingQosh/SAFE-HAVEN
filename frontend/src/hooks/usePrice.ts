/**
 * usePrice Hook — Access token prices from context
 * Throws if PriceProvider is not in component tree
 */

import { useContext } from 'react'
import { PriceContext } from '../context/PriceContext'

export function usePrice() {
  const context = useContext(PriceContext)

  if (!context) {
    throw new Error(
      'usePrice must be used within a PriceProvider. ' +
      'Ensure PriceProvider wraps your component tree.'
    )
  }

  return context
}
