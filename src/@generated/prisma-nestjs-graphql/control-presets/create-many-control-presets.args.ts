import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsCreateManyInput } from './control-presets-create-many.input';

@ArgsType()
export class CreateManyControlPresetsArgs {

    @Field(() => [ControlPresetsCreateManyInput], {nullable:false})
    data!: Array<ControlPresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
