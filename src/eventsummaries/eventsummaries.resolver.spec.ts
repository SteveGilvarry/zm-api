import { Test, TestingModule } from '@nestjs/testing';
import { EventsummariesResolver } from './eventsummaries.resolver';
import { EventsummariesService } from './eventsummaries.service';

describe('EventsummariesResolver', () => {
  let resolver: EventsummariesResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [EventsummariesResolver, EventsummariesService],
    }).compile();

    resolver = module.get<EventsummariesResolver>(EventsummariesResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
