package geneuron.neuron

trait Neuron {
  def process(in: Array[Double]): Array[Double]
}
