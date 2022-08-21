import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsCreateInput } from './stats-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneStatsArgs {

    @Field(() => StatsCreateInput, {nullable:false})
    @Type(() => StatsCreateInput)
    data!: StatsCreateInput;
}
