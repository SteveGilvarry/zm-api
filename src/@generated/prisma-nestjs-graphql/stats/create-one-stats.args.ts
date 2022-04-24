import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsCreateInput } from './stats-create.input';

@ArgsType()
export class CreateOneStatsArgs {

    @Field(() => StatsCreateInput, {nullable:false})
    data!: StatsCreateInput;
}
