import { Test, TestingModule } from '@nestjs/testing';
import { MontagelayoutsService } from './montagelayouts.service';

describe('MontagelayoutsService', () => {
  let service: MontagelayoutsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MontagelayoutsService],
    }).compile();

    service = module.get<MontagelayoutsService>(MontagelayoutsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
