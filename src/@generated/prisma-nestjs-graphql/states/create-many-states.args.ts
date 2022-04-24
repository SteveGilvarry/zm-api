import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesCreateManyInput } from './states-create-many.input';

@ArgsType()
export class CreateManyStatesArgs {

    @Field(() => [StatesCreateManyInput], {nullable:false})
    data!: Array<StatesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
