//
// Created by davide on 7/26/19.
//

#ifndef THREADSTREAM
#define THREADSTREAM

#include <iostream>
#include <mutex>
#include <sstream>

#define serr SyncCout(std::cerr)
#define sout SyncCout(std::cout)

/**
 * Thread-safe std::ostream class.
 * https://bit.ly/2OijbYd
 *
 * Usage:
 *    sout << "Hello world!" << std::endl;
 *    serr << "Hello world!" << std::endl;
 */
class SyncCout : public std::ostringstream
{
public:
  explicit SyncCout(std::ostream& os) : os_(os)
  {
    // copyfmt causes odd problems with lost output
    // probably some specific flag
    //            copyfmt(os);
    // copy whatever properties are relevant
    imbue(os.getloc());
    precision(os.precision());
    width(os.width());
    setf(std::ios::fixed, std::ios::floatfield);
  }

  ~SyncCout() override
  {
    std::lock_guard<std::mutex> guard(_mutex_threadstream);
    os_ << this->str();
  }

private:
  static std::mutex _mutex_threadstream;
  std::ostream& os_;
};

std::mutex SyncCout::_mutex_threadstream{};

#endif
