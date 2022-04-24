import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsUpdateManyMutationInput } from './stats-update-many-mutation.input';
import { StatsWhereInput } from './stats-where.input';

@ArgsType()
export class UpdateManyStatsArgs {

    @Field(() => StatsUpdateManyMutationInput, {nullable:false})
    data!: StatsUpdateManyMutationInput;

    @Field(() => StatsWhereInput, {nullable:true})
    where?: StatsWhereInput;
}
