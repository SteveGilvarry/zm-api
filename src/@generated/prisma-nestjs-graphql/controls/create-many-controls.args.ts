import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsCreateManyInput } from './controls-create-many.input';

@ArgsType()
export class CreateManyControlsArgs {

    @Field(() => [ControlsCreateManyInput], {nullable:false})
    data!: Array<ControlsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
