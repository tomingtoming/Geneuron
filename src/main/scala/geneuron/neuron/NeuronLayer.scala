package geneuron.neuron

import breeze.linalg.DenseMatrix
import breeze.numerics._

class NeuronLayer(ins: Int, outs: Int, array: Array[Double]) extends Neuron {
  if (array.length != outs * (ins + 1)) throw new IllegalArgumentException("Requirement: array length equals outs*(ins+1)")
  val omega = new DenseMatrix[Double](ins, outs, array, 0)
  val bias = new DenseMatrix[Double](1, outs, array, ins * outs)
  val inMx = new DenseMatrix[Double](1, ins)
  override def process(in: Array[Double]): Array[Double] = {
    Array.copy(in, 0, inMx.data, 0, ins)
    (1.0 / (exp((inMx * omega + bias) * -1.0) + 1.0)).data
  }
}
