package geneuron.neuron

class NeuronCluster(layers: Seq[Int], array: Array[Double]) extends Neuron {
  if (layers.length < 2) throw new IllegalArgumentException("Requirement: NeuronCluster must have more than 2 layers")
  val neuronLayers: Seq[NeuronLayer] = (layers zip layers.tail).foldLeft[(Array[Double], List[NeuronLayer])](array, Nil) {
    case ((ary, list), (ins, outs)) =>
      val consumeCount = outs * (ins + 1)
      val neuronLayer = new NeuronLayer(ins, outs, ary.take(consumeCount))
      (ary.drop(consumeCount), neuronLayer :: list)
  }._2.reverse

  override def process(in: Array[Double]): Array[Double] = {
    neuronLayers.foldLeft(in) { (in, neuronLayer) =>
      neuronLayer.process(in)
    }
  }
}
