import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';

@ArgsType()
export class FindUniqueMonitorsArgs {

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    where!: MonitorsWhereUniqueInput;
}
