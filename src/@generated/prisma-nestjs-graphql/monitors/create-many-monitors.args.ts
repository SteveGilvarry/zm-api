import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsCreateManyInput } from './monitors-create-many.input';

@ArgsType()
export class CreateManyMonitorsArgs {

    @Field(() => [MonitorsCreateManyInput], {nullable:false})
    data!: Array<MonitorsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
