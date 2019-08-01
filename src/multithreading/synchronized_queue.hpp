//
// Created by davide on 6/17/19.
//

#ifndef __SYNCHRONIZED_QUEUE_HPP__
#define __SYNCHRONIZED_QUEUE_HPP__

#include "spinlock.hpp"
#include <queue>

/**
 * \brief Wrapper for a thread-safe FIFO queue
 * \tparam T Type of element.
 *
 * A wrapper for the std::queue class providing thread-safe capabilities. Due to
 * this additional constraint, it is not possible to just lookup the first
 * element: as soon as an element is retrieved it is also popped from the queue
 */
template<typename T>
class SynchronizedQueue
{
public:
  /**
   * \brief Default constructor
   */
  SynchronizedQueue() = default;

  /**
   * \brief Default destructor
   */
  ~SynchronizedQueue() = default;

  /**
   * \brief Checks if the queue is empty
   * \return true if the queue is empty
   */
  bool empty()
  {
    spin.lock();
    bool retval = container.empty();
    spin.unlock();
    return retval;
  }

  /**
   * \brief Returns the number of elements in the queue
   * \return the number of elements in the queue
   */
  size_t size()
  {
    spin.lock();
    size_t retval = container.size();
    spin.unlock();
    return retval;
  }

  /**
   * \brief Access and removes the next element
   * \return the first element of the queue
   */
  T front()
  {
    T retval{0};
    spin.lock();
    if(!container.empty())
    {
      retval = container.front();
      container.pop();
    }
    spin.unlock();
    return retval;
  }

  /**
   * \brief Adds an element to the back of the queue
   * \param[in] val The element that will be added to the queue
   */
  void push(const T& val)
  {
    spin.lock();
    container.push(val);
    spin.unlock();
  }

  /**
   * \brief Adds an element in-place at the end of the queue
   * Adds a new element at the end of the queue, after its current last
   * element. This new element is constructed in place passing args as the
   * arguments for its constructor
   * \param[in] args the arguments of the constructor
   */
  template<class... Args>
  void emplace(Args&&... args)
  {
    spin.lock();
    container.emplace(args...);
    spin.unlock();
  }

private:
  // spinlock to ensure thread safety (all operations are fast, so mutexes are
  // unnecessary)
  Spinlock spin;

  // actual queue performing the storage
  std::queue<T> container;
};

#endif //__SYNCHRONIZED_QUEUE_HPP__
